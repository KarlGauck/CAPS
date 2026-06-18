use std::ops;

#[derive(Clone, Copy)]
pub struct Vec4 {
    pub x0: f64,
    pub x1: f64,
    pub x2: f64,
    pub x3: f64,
}

impl Vec4 {
    pub fn new(x0: f64, x1: f64, x2: f64, x3: f64) -> Self {
        Self { x0, x1, x2, x3 }
    }
}

impl ops::Add<Vec4> for Vec4 {
    type Output = Vec4;

    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self.x0 + rhs.x0, self.x1 + rhs.x1, self.x2 + rhs.x2, self.x3 + rhs.x3)
    }
}

impl ops::Add<f64> for Vec4 {
    type Output = Vec4;

    fn add(self, rhs: f64) -> Vec4 {
        Vec4::new(self.x0 + rhs, self.x1 + rhs, self.x2 + rhs, self.x3 + rhs)
    }
}

impl ops::Add<Vec4> for f64 {
    type Output = Vec4;

    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self + rhs.x0, self + rhs.x1, self + rhs.x2, self + rhs.x3)
    }
}

impl ops::Sub<Vec4> for Vec4 {
    type Output = Vec4;

    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self.x0 - rhs.x0, self.x1 - rhs.x1, self.x2 - rhs.x2, self.x3 - rhs.x3)
    }
}

impl ops::Sub<f64> for Vec4 {
    type Output = Vec4;

    fn sub(self, rhs: f64) -> Vec4 {
        Vec4::new(self.x0 - rhs, self.x1 - rhs, self.x2 - rhs, self.x3 - rhs)
    }
}

impl ops::Sub<Vec4> for f64 {
    type Output = Vec4;

    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self - rhs.x0, self - rhs.x1, self - rhs.x2, self - rhs.x3)
    }
}

impl ops::Mul<f64> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: f64) -> Vec4 {
        Vec4::new(self.x0 * rhs, self.x1 * rhs, self.x2 * rhs, self.x3 * rhs)
    }
}

impl ops::Mul<Vec4> for f64 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self * rhs.x0, self * rhs.x1, self * rhs.x2, self * rhs.x3)
    }
}

impl ops::Div<f64> for Vec4 {
    type Output = Vec4;

    fn div(self, rhs: f64) -> Vec4 {
        Vec4::new(self.x0 / rhs, self.x1 / rhs, self.x2 / rhs, self.x3 / rhs)
    }
}

impl ops::Div<Vec4> for f64 {
    type Output = Vec4;

    fn div(self, rhs: Vec4) -> Vec4 {
        Vec4::new(self / rhs.x0, self / rhs.x1, self / rhs.x2, self / rhs.x3)
    }
}
