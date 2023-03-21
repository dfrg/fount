pub fn floor(x: i32) -> i32 {
    x & !63
}

pub fn round(x: i32) -> i32 {
    floor(x + 32)
}

#[inline(always)]
pub fn mul(a: i32, b: i32) -> i32 {
    let ab = a as i64 * b as i64;
    ((ab + 0x8000 - i64::from(ab < 0)) >> 16) as i32
}

pub fn div(mut a: i32, mut b: i32) -> i32 {
    let mut sign = 1;
    if a < 0 {
        a = -a;
        sign = -1;
    }
    if b < 0 {
        b = -b;
        sign = -sign;
    }
    let q = if b == 0 {
        0x7FFFFFFF
    } else {
        ((((a as u64) << 16) + ((b as u64) >> 1)) / (b as u64)) as u32
    };
    if sign < 0 {
        -(q as i32)
    } else {
        q as i32
    }
}

pub fn muldiv(mut a: i32, mut b: i32, mut c: i32) -> i32 {
    let mut sign = 1;
    if a < 0 {
        a = -a;
        sign = -1;
    }
    if b < 0 {
        b = -b;
        sign = -sign;
    }
    if c < 0 {
        c = -c;
        sign = -sign;
    }
    let d = if c > 0 {
        ((a as i64) * (b as i64) + ((c as i64) >> 1)) / c as i64
    } else {
        0x7FFFFFFF
    };
    if sign < 0 {
        -(d as i32)
    } else {
        d as i32
    }
}

pub fn ceil(x: i32) -> i32 {
    floor(x + 63)
}

pub fn floor_pad(x: i32, n: i32) -> i32 {
    x & !(n - 1)
}

pub fn round_pad(x: i32, n: i32) -> i32 {
    floor_pad(x + n / 2, n)
}

pub fn muldiv_no_round(mut a: i32, mut b: i32, mut c: i32) -> i32 {
    let mut s = 1;
    if a < 0 {
        a = -a;
        s = -1;
    }
    if b < 0 {
        b = -b;
        s = -s;
    }
    if c < 0 {
        c = -c;
        s = -s;
    }
    let d = if c > 0 {
        ((a as i64) * (b as i64)) / c as i64
    } else {
        0x7FFFFFFF
    };
    if s < 0 {
        -(d as i32)
    } else {
        d as i32
    }
}

pub fn mul14(a: i32, b: i32) -> i32 {
    let mut v = a as i64 * b as i64;
    v += 0x2000 + (v >> 63);
    (v >> 14) as i32
}

pub fn dot14(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    let mut v1 = ax as i64 * bx as i64;
    let v2 = ay as i64 * by as i64;
    v1 += v2;
    v1 += 0x2000 + (v1 >> 63);
    (v1 >> 14) as i32
}
