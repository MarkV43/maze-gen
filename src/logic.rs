use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Clone, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<Dir> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dir {
        match rng.gen_range(0..4) {
            0 => Dir::Up,
            1 => Dir::Down,
            2 => Dir::Left,
            3 => Dir::Right,
            _ => unreachable!(),
        }
    }
}

pub struct Maze {
    width: u32,
    height: u32,
    origin: (u32, u32),
    tiles: Vec<Option<Dir>>,
    horz_walls: Vec<bool>,
    vert_walls: Vec<bool>,
}

impl Maze {
    pub fn new(width: u32, height: u32) -> Self {
        let mut tiles = vec![Some(Dir::Left); (width * height) as usize];

        tiles
            .chunks_exact_mut(width as usize)
            .enumerate()
            .for_each(|(i, row)| {
                row[0] = if i == 0 { None } else { Some(Dir::Up) };
            });

        let mut horz_walls = vec![true; (width * (height - 1)) as usize];
        let vert_walls = vec![false; ((width - 1) * height) as usize];

        horz_walls
            .chunks_exact_mut(width as usize)
            .for_each(|row| row[0] = false);

        Self {
            width,
            height,
            tiles,
            horz_walls,
            vert_walls,
            origin: (0, 0),
        }
    }

    pub fn step(&mut self, rng: &mut impl Rng) {
        let mut new_origin;
        let mut direction;
        loop {
            direction = rng.gen();
            new_origin = match direction {
                Dir::Left if self.origin.0 == 0 => None,
                Dir::Left => Some((self.origin.0 - 1, self.origin.1)),
                Dir::Up if self.origin.1 == 0 => None,
                Dir::Up => Some((self.origin.0, self.origin.1 - 1)),
                Dir::Right if self.origin.0 + 1 == self.width => None,
                Dir::Right => Some((self.origin.0 + 1, self.origin.1)),
                Dir::Down if self.origin.1 + 1 == self.height => None,
                Dir::Down => Some((self.origin.0, self.origin.1 + 1)),
            };

            if new_origin.is_some() {
                break;
            }
        }

        let old_origin = self.origin;
        let new_origin = new_origin.unwrap();
        let direction = direction;

        match direction {
            Dir::Up => {
                self.horz_walls[(new_origin.0 + new_origin.1 * self.width) as usize] = false;
            }
            Dir::Down => {
                self.horz_walls[(old_origin.0 + old_origin.1 * self.width) as usize] = false;
            }
            Dir::Left => {
                self.vert_walls[(new_origin.0 + new_origin.1 * (self.width - 1)) as usize] = false;
            }
            Dir::Right => {
                self.vert_walls[(old_origin.0 + old_origin.1 * (self.width - 1)) as usize] = false;
            }
        }

        let other_dir = self.tiles[(new_origin.0 + new_origin.1 * self.width) as usize]
            .as_ref()
            .unwrap();
        match (&direction, other_dir) {
            (Dir::Down, Dir::Up) => {}
            (_, Dir::Up) => {
                self.horz_walls[(new_origin.0 + (new_origin.1 - 1) * self.width) as usize] = true;
            }
            (Dir::Up, Dir::Down) => {}
            (_, Dir::Down) => {
                self.horz_walls[(new_origin.0 + new_origin.1 * self.width) as usize] = true;
            }
            (Dir::Right, Dir::Left) => {}
            (_, Dir::Left) => {
                self.vert_walls[((new_origin.0 - 1) + new_origin.1 * (self.width - 1)) as usize] =
                    true;
            }
            (Dir::Left, Dir::Right) => {}
            (_, Dir::Right) => {
                self.vert_walls[(new_origin.0 + new_origin.1 * (self.width - 1)) as usize] = true;
            }
        }

        self.tiles[(old_origin.0 + old_origin.1 * self.width) as usize] = Some(direction);
        self.tiles[(new_origin.0 + new_origin.1 * self.width) as usize] = None;

        self.origin = new_origin;
    }

    pub fn init(&mut self, rng: &mut impl Rng) {
        for _ in 0..(self.width as u64) * (self.height as u64) * 10 {
            self.step(rng);
        }
    }

    pub fn to_str(&self, dirs: bool) -> String {
        let mut result = String::new();

        for y in 0..(self.height as usize * 2 + 1) {
            let ye = y % 2 == 0;
            let yy = y / 2;
            for x in 0..(self.width as usize * 2 + 1) {
                let xe = x % 2 == 0;
                let xx = x / 2;
                match (xe, ye) {
                    (false, false) => {
                        result += if dirs {
                            match self.tiles[xx + yy * self.width as usize] {
                                Some(Dir::Up) => "^",
                                Some(Dir::Down) => "v",
                                Some(Dir::Left) => "<",
                                Some(Dir::Right) => ">",
                                None => "X",
                            }
                        } else {
                            " "
                        }
                    }
                    (true, true) => result += "+",
                    (false, true) if yy == 0 || yy == self.height as usize => result += "-",
                    (false, true) => {
                        result += if self.horz_walls[xx + (yy - 1) * self.width as usize] {
                            "-"
                        } else {
                            " "
                        }
                    }
                    (true, false) if xx == 0 || xx == self.width as usize => result += "|",
                    (true, false) => {
                        result += if self.vert_walls[(xx - 1) + yy * (self.width - 1) as usize] {
                            "|"
                        } else {
                            " "
                        }
                    }
                }
            }

            result += "\n";
        }

        result
    }
}
