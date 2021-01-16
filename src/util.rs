#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

impl Vec2 {
    pub const fn zero() -> Self {
        Vec2 { x: 0, y: 0 }
    }
}

impl Rect {
    pub const fn right(&self) -> i32 {
        let right = self.left + self.width - 1;
        right
    }

    pub const fn bottom(&self) -> i32 {
        let bottom = self.top + self.height - 1;
        bottom
    }

    pub const fn center_x(&self) -> i32 {
        self.left + self.width / 2
    }

    pub const fn center_y(&self) -> i32 {
        self.top + self.height / 2
    }
}
