use std::fmt;

pub struct Bytes(i64);

impl Bytes {
    pub fn new(n: i64) -> Self {
        Self(n)
    }
}

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < 1024 {
            write!(f, "{} B", self.0)
        } else if self.0 < 1024 * 1024 {
            write!(f, "{:.2} kB", self.0 as f64 / 1024.0)
        } else if self.0 < 1024 * 1024 * 1024 {
            write!(f, "{:.2} MB", self.0 as f64 / 1024.0 / 1024.0)
        } else {
            write!(f, "{:.2} GB", self.0 as f64 / 1024.0 / 1024.0 / 1024.0)
        }
    }
}

pub struct Duration(u64);

impl Duration {
    pub fn from_secs(n: u64) -> Self {
        Self(n)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "P")?;

        let mut n: u64 = self.0;

        if n >= 3600 * 24 {
            write!(f, "D{}", n / (3600 * 24))?;
            n %= 3600 * 24;
        }

        if n > 0 {
            write!(f, "T")?;
        }

        if n >= 3600 {
            write!(f, "H{:02}", n / 3600)?;
        }
        n %= 3600;
        if n >= 60 {
            write!(f, "M{:02}", n / 60)?;
        }
        n %= 60;
        if n > 0 {
            write!(f, "S{:02}", n)?;
        }

        Ok(())
    }
}
