use iced::{mouse, touch, Color, Size};
use iced::mouse::{Cursor, Interaction};
use iced::widget::canvas::{Cache, Canvas, Geometry, Path, Stroke, event};
use iced::{Element, Rectangle, Renderer, Sandbox, Settings, Theme, Point, Length};
use iced::widget::{canvas, column, container};

fn main() -> iced::Result {
    GomokuGame::run(Settings::default())
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ClickBoard(usize),
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum GameState {
    WaitBlack,
    WaitWhite,
    CheckBlack,
    CheckWhite,
}

struct GomokuGame {
    board: Board,
    state: GameState,
}

impl Sandbox for GomokuGame {
    type Message = Message;

    fn new() -> Self {
        Self {
            board: Board::default(),
            state: GameState::WaitBlack,
        }
    }

    fn title(&self) -> String {
        String::from("Gomoku")
    }

    fn update(&mut self, message: Self::Message) {
        let mut next_state = None;
        match message {
            Self::Message::ClickBoard(index) => {
                println!("Message ClickBoard at {}, current state {:?}", index, self.state);
                if self.board.is_empty_at(index) {
                    match self.state {
                        GameState::WaitBlack => {
                            println!("Put black chess at {}", index);
                            self.board.put_chess(index, true);
                            next_state = Some(GameState::CheckBlack);
                        },
                        GameState::WaitWhite => {
                            println!("Put white chess at {}", index);
                            self.board.put_chess(index, false);
                            next_state = Some(GameState::CheckWhite);
                        },
                        _ => ()
                    };
                }
            },
        };

        match next_state {
            Some(GameState::CheckBlack) => { self.state = GameState::WaitWhite; }
            Some(GameState::CheckWhite) => { self.state = GameState::WaitBlack; }
            _ => ()
        };
    }

    fn view(&self) -> Element<'_, Self::Message> {
       let content = column![self.board.view()];
       container(content).into()
    }
}

#[derive(PartialEq)]
enum ChessColor {
    Black,
    White,
}

struct Chess {
    pos: Point<usize>,
    color: ChessColor,
}

#[derive(PartialEq, Copy, Clone)]
enum CellState {
    Empty,
    Black,
    White,
}

struct Board {
    padding: f32,
    cell_size: f32,
    chess_size: f32,
    line_width: f32,
    grid_size: f32,
    cells_per_row: usize,
    cells: Vec<CellState>,
    chesses: Vec<Chess>,
    chesses_cache: Cache,
    grid_cache: Cache,
    overlay_cache: Cache,
}

impl Board {
    fn new(padding: f32, cell_size: f32, chess_size: f32, line_width: f32) -> Self {
        let cells_per_row = 15;
        let grid_size = (cells_per_row - 1) as f32 * cell_size + line_width;
        let mut cells = Vec::with_capacity(cells_per_row * cells_per_row);
        cells.resize(cells_per_row * cells_per_row, CellState::Empty);

        Self {
            padding,
            cell_size,
            chess_size,
            line_width,
            grid_size,
            cells_per_row,
            cells,
            chesses: vec![],
            chesses_cache: Cache::default(),
            grid_cache: Cache::default(),
            overlay_cache: Cache::default(),
        }
    }

    fn valid_index(&self, index: usize) -> bool {
        index < self.cells_per_row * self.cells_per_row
    }

    fn valid_pos(&self, col: usize, row: usize) -> bool {
        col <= self.cells_per_row && row <= self.cells_per_row
    }

    fn index_to_pos(&self, index: usize) -> Point<usize> {
        Point::new(index % self.cells_per_row, index / self.cells_per_row)
    }

    fn pos_to_index(&self, pos: Point<usize>) -> usize {
        pos.x + pos.y * self.cells_per_row
    }

    fn is_empty_at(&self, index: usize) -> bool {
        self.valid_index(index) && self.cells[index] == CellState::Empty
    }

    fn put_chess(&mut self, index: usize, is_black: bool) {
        if self.valid_index(index) {
            let grid_pos = self.index_to_pos(index);
            self.chesses.push(Chess {pos: grid_pos, color: if is_black { ChessColor::Black } else { ChessColor::White } });
            self.cells[index] = if is_black { CellState::Black } else { CellState::White };
            self.chesses_cache.clear();
            self.overlay_cache.clear();
        } else {
            panic!("Index out of range when putting chess, max is {}, but got {}.", self.cells_per_row * self.cells_per_row, index);
        }
    }

    fn remove_last_chess(&mut self) {
        match self.chesses.pop() {
            Some(chess) => {
                let index = self.pos_to_index(chess.pos);
                self.cells[index] = CellState::Empty;
                self.chesses_cache.clear();
                self.overlay_cache.clear();
            },
            None => { panic!("Cannot remove last chess because chesses is empty."); }
        }
    }

    fn clear(&mut self) {
        *self = Self::new(self.padding, self.cell_size, self.chess_size, self.line_width);
    }

    fn view(&self) -> Element<Message> {
        Canvas::new(self).width(Length::Fill).height(Length::Fill).into()
    }

    fn grid_pos(&self, x: f32, y: f32, dis_scale: f32) -> Option<Point<usize>> {
        let pos_from_grid = Point::new(x - self.padding, y - self.padding);
        let col = (pos_from_grid.x / self.cell_size).round() as i32;
        let row = (pos_from_grid.y / self.cell_size).round() as i32;
        if col > 0 && row > 0 && self.valid_pos(col as usize, row as usize) {
            let dis = pos_from_grid.distance(Point::new(col as f32 * self.cell_size, row as f32 * self.cell_size));
            // println!("board pos {}, grid pos {}, col {}, row {}, dis {}", Point::new(x, y), pos_from_grid, col, row, dis);
            if dis * 2.0 > self.cell_size * dis_scale { None } else { Some(Point::new(col as usize, row as usize)) }
        } else {
            None
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new(45.0, 48.0, 42.0, 2.0)
    }
}

impl canvas::Program<Message> for Board {
    type State = ();
    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let on_click = || {
            match cursor.position_in(bounds) {
                Some(pos) => {
                    match self.grid_pos(pos.x, pos.y, 0.6) {
                        Some(grid_pos) => {
                            println!("Press at board {}, try to put chess at index {}", grid_pos, self.pos_to_index(grid_pos));
                            (event::Status::Captured, Some(Message::ClickBoard(self.pos_to_index(grid_pos))))
                        },
                        None => (canvas::event::Status::Captured, None),
                    }
                },
                None => (canvas::event::Status::Captured, None),
            }
        };

        match event {
            canvas::Event::Touch(touch::Event::FingerPressed { .. }) => { on_click() },
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => { on_click() },
            _ => (canvas::event::Status::Captured, None),
        }
    }

    fn draw(
        &self,
        _interaction: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor
    ) -> Vec<Geometry> {
        // println!("board draw called, already have {} chesses", self.chesses.len());

        let grid = self.grid_cache.draw(renderer, bounds.size(), |frame| {
            let bg_color = Color::from_rgb8(0xf0, 0xf0, 0xf0);
            let grid_color = Color::from_rgb8(0x60, 0x64, 0x6b);
            frame.fill_rectangle(bounds.position(), bounds.size(), bg_color);
            for row in 0..self.cells_per_row {
                frame.fill_rectangle(
                    Point::new(self.padding, self.padding + row as f32 * self.cell_size),
                    Size::new(self.grid_size, self.line_width as f32),
                    grid_color,
                );
                frame.fill_rectangle(
                    Point::new(self.padding + row as f32 * self.cell_size, self.padding),
                    Size::new(self.line_width as f32, self.grid_size),
                    grid_color,
                );
            }
        });

        let chesses = self.chesses_cache.draw(renderer, bounds.size(), |frame| {
            // TODO: read from config
            let outer_color = Color::from_rgb8(0x60, 0x60, 0x60);
            let black_chess_color = Color::from_rgb8(0x20, 0x20, 0x20);
            let white_chess_color = Color::from_rgb8(0xf0, 0xf0, 0xf0);
            for c in self.chesses.iter() {
                let chess_center = Point::new(
                    self.padding + c.pos.x as f32 * self.cell_size,
                    self.padding + c.pos.y as f32 * self.cell_size);
                let chess_color = if c.color == ChessColor::Black { black_chess_color } else { white_chess_color };
                frame.fill(&Path::circle(chess_center, self.chess_size / 2.0), outer_color);
                frame.fill(&Path::circle(chess_center, self.chess_size / 2.0 - self.line_width), chess_color);
            }
        });

        let overlay = self.overlay_cache.draw(renderer, bounds.size(), |frame| {
            match self.chesses.last() {
                Some(last_chess) => {
                    let cross_half_size = self.cell_size / 7.0;
                    let chess_center = Point::new(
                        self.padding + last_chess.pos.x as f32 * self.cell_size,
                        self.padding + last_chess.pos.y as f32 * self.cell_size);
                    let cross = Path::new(|b| {
                        b.move_to(Point::new(chess_center.x - cross_half_size, chess_center.y));
                        b.line_to(Point::new(chess_center.x + cross_half_size, chess_center.y));
                        b.move_to(Point::new(chess_center.x, chess_center.y - cross_half_size));
                        b.line_to(Point::new(chess_center.x, chess_center.y + cross_half_size));
                    });
                    frame.stroke(
                        &cross,
                        Stroke::default()
                            .with_color(Color::from_rgb8(0xff, 0x00, 0x00))
                            .with_width(self.line_width as f32));
                },
                None => ()
            }
        });
        vec![grid, chesses, overlay]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Interaction {
        match cursor.position_in(bounds) {
            Some(pos) => {
                match self.grid_pos(pos.x, pos.y, 0.6) {
                    Some(_) => Interaction::Pointer,
                    None => Interaction::default(),
                }
            },
            None => Interaction::default(),
        }
    }
}

