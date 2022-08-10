const BOARD_WIDTH: usize = 100;
const BOARD_HEIGHT: usize = 100;
const BOARD_SIZE: usize = BOARD_WIDTH * BOARD_HEIGHT;
// pub type Board = [u8; BOARD_SIZE];

pub struct Board {
    grid: [(u8, bool); BOARD_SIZE],
}

impl Board {
    pub fn new() -> Board {
        let mut b = Board {
            grid: [(0, true); BOARD_SIZE],
        };
        for i in 0..BOARD_SIZE {
            b.grid[i] = (rand::random::<u8>(), rand::random::<bool>());
        }
        for i in 0..BOARD_SIZE {
            let mut dir = b.grid[i].1;
            let mut val = b.grid[i].0;
            if i != 0 && i != BOARD_SIZE - 1 {
                let side = rand::random::<u8>() % 3;
                dir = if side == 0 {
                    b.grid[i - 1].1
                } else if side == 1 {
                    b.grid[i].1
                } else {
                    b.grid[i + 1].1
                };

                val = if side == 0 {
                    (b.grid[i - 1].0 / 2 + b.grid[i].0 / 2)
                } else if side == 1 {
                    b.grid[i].0
                } else {
                    (b.grid[i + 1].0 / 2 + b.grid[i].0 / 2)
                };
            }

            b.grid[i] = (val, dir);
        }
        b
    }

    pub fn update_board(&mut self) {
        for i in 0..BOARD_SIZE {
            if self.grid[i].1 {
                //increasing
                self.grid[i] = if self.grid[i].0 == 255 {
                    (255, false)
                } else {
                    (self.grid[i].0 + 1, true)
                };
            } else {
                //decreasing
                self.grid[i] = if self.grid[i].0 == 0 {
                    (0, true)
                } else {
                    (self.grid[i].0 - 1, false)
                };
            }
        }
    }
}
