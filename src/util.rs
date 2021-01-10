pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn right(&self) -> i32 {
        let right = self.left + self.width - 1;
        assert!(self.left < right);
        right
    }

    pub fn bottom(&self) -> i32 {
        let bottom = self.top + self.height - 1;
        assert!(self.top < bottom);
        bottom
    }

    pub fn center_x(&self) -> i32 {
        self.left + self.width / 2
    }

    pub fn center_y(&self) -> i32 {
        self.top + self.height / 2
    }
}
