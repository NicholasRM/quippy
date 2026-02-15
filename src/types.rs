use std::collections::HashMap;

pub type QObject = HashMap<String, QType>;

#[derive(Clone, Debug)]
pub enum QType {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Void,
    Err,
    List(Vec<QType>),
    Obj(QObject),
    Thread(Option<usize>),
    Func(QObject, ()),
}

impl QType {
    pub fn like(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Void, Self::Void) => Self::Bool(true),
            (Self::Err, Self::Err) => Self::Bool(true),
            (Self::Int(_), Self::Int(_)) => Self::Bool(true),
            (Self::Float(_), Self::Float(_)) => Self::Bool(true),
            (Self::Bool(_), Self::Bool(_)) => Self::Bool(true),
            (Self::Str(_), Self::Str(_)) => Self::Bool(true),
            (Self::List(_), Self::List(_)) => Self::Bool(true),
            (Self::Obj(_), Self::Obj(_)) => Self::Bool(true),
            (Self::Func(_, _), Self::Func(_, _)) => Self::Bool(true),
            (Self::Thread(_), Self::Thread(_)) => Self::Bool(true),
            _ => Self::Bool(false),
        }
    }

    pub fn into(lhs: Self, rhs: Self) -> Self {
        if let Self::Bool(true) = Self::like(lhs.clone(), rhs.clone()) {
            lhs
        } else {
            match rhs {
                Self::Void => Self::Void,
                Self::Err => Self::Err,
                Self::Func(_, _) => Self::Err,
                Self::Thread(_) => Self::Err,
                Self::Int(_) => Self::to_int(lhs),
                Self::Float(_) => Self::to_float(lhs),
                Self::Bool(_) => Self::to_bool(lhs),
                Self::Str(_) => Self::to_str(lhs),
                Self::List(_) => Self::to_list(lhs),
                Self::Obj(_) => Self::to_obj(lhs),
            }
        }
    }

    fn to_int(expr: Self) -> Self {
        match expr {
            Self::Bool(b) => Self::Int(b as i64),
            Self::Float(f) => Self::Int(f as i64),
            Self::Str(s) => {
                if let Ok(i) = s.parse::<i64>() {
                    Self::Int(i)
                } else {
                    Self::Err
                }
            }
            _ => Self::Err,
        }
    }

    fn to_float(expr: Self) -> Self {
        match expr {
            Self::Bool(b) => Self::Float(b as i64 as f64),
            Self::Int(i) => Self::Float(i as f64),
            Self::Str(s) => {
                if let Ok(f) = s.parse::<f64>() {
                    Self::Float(f)
                } else {
                    Self::Err
                }
            }
            _ => Self::Err,
        }
    }

    fn to_bool(expr: Self) -> Self {
        match expr {
            Self::Int(i) => Self::Bool(i != 0),
            Self::Float(f) => Self::Bool(f != 0.0),
            Self::Str(s) => Self::Bool(s.len() != 0),
            Self::Void => Self::Bool(true),
            Self::Err => Self::Bool(false),
            Self::List(l) => Self::Bool(l.len() != 0),
            Self::Obj(o) => Self::Bool(o.keys().count() != 0),
            _ => Self::Err,
        }
    }

    fn to_list(expr: Self) -> Self {
        match expr {
            Self::Str(s) => Self::List(s.bytes().map(|b| Self::Int(b as i64)).collect()),
            Self::Obj(o) => Self::List(o.into_keys().map(|k| from_qobj_key(k)).collect()),
            _ => Self::Err,
        }
    }

    fn to_obj(expr: Self) -> Self {
        match expr {
            Self::Str(s) => Self::Obj(
                s.bytes()
                    .enumerate()
                    .map(|(i, b)| ((i as i64).to_string(), Self::Int(b as i64)))
                    .collect(),
            ),
            Self::List(l) => Self::Obj(
                l.into_iter()
                    .enumerate()
                    .map(|(i, e)| ((i as i64).to_string(), e))
                    .collect(),
            ),
            _ => Self::Err,
        }
    }

    fn to_str(expr: Self) -> Self {
        match expr {
            Self::Int(i) => Self::Str(i.to_string()),
            Self::Float(f) => Self::Str(f.to_string()),
            Self::Bool(b) => Self::Str(b.to_string()),
            Self::Void => Self::Str("()".to_string()),
            Self::Err => Self::Str("err".to_string()),
            Self::Func(_, _) => Self::Str("\\(...) => ...".to_string()),
            Self::Thread(None) => Self::Str("@this".to_string()),
            Self::Thread(Some(t)) => Self::Str(format!("@{}", t)),
            Self::List(l) => {
                let mut s = String::from("[");
                s.extend(
                    l.into_iter()
                        .map(|q| {
                            let Self::Str(e) = Self::to_str(q) else {
                                unreachable!()
                            };
                            e
                        })
                        .collect::<Vec<String>>(),
                );
                Self::Str(s + "]")
            }
            Self::Obj(o) => {
                let mut s = String::from("{");
                if o.keys().count() == 0 {
                    return Self::Str(s + "}");
                }
                s += " ";
                s.extend(
                    o.into_iter()
                        .map(|(k, v)| {
                            let Self::Str(value) = Self::to_str(v) else {
                                unreachable!()
                            };
                            match from_qobj_key(k) {
                                Self::Str(key) => format!("\"{}\": {}", key, value),
                                Self::Int(key) => format!("{}: {}", key, value),
                                _ => unreachable!(),
                            }
                        })
                        .collect::<Vec<String>>(),
                );
                Self::Str(s + " }")
            }
            s => s,
        }
    }
}

impl QType {
    pub fn add(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Int(l.wrapping_add(r)),
            (Self::Float(l), Self::Float(r)) => Self::Float(l + r),
            (Self::Str(l), Self::Str(r)) => Self::Str(format!("{}{}", l.clone(), r.clone())),
            (Self::List(l), Self::List(r)) => Self::List({
                let mut list = l.clone();
                list.extend(r.clone());
                list
            }),
            (Self::Obj(l), Self::Obj(r)) => Self::Obj({
                let mut obj = l.clone();
                obj.extend(r.clone());
                obj
            }),
            _ => Self::Err,
        }
    }

    pub fn sub(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Int(l.wrapping_sub(r)),
            (Self::Float(l), Self::Float(r)) => Self::Float(l - r),
            _ => Self::Err,
        }
    }

    pub fn mul(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Int(l.wrapping_mul(r)),
            (Self::Float(l), Self::Float(r)) => Self::Float(l * r),
            _ => Self::Err,
        }
    }

    pub fn div(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Int(l.wrapping_div(r)),
            (Self::Float(l), Self::Float(r)) => Self::Float(l / r),
            _ => Self::Err,
        }
    }

    pub fn modulo(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Int(l.wrapping_rem(r)),
            (Self::Float(l), Self::Float(r)) => Self::Float(l % r),
            _ => Self::Err,
        }
    }

    pub fn and(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Int(l & r),
            (Self::Bool(l), Self::Bool(r)) => Self::Bool(l && r),
            _ => Self::Err,
        }
    }

    pub fn or(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Int(l | r),
            (Self::Bool(l), Self::Bool(r)) => Self::Bool(l || r),
            _ => Self::Err,
        }
    }

    pub fn xor(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Int(l ^ r),
            (Self::Bool(l), Self::Bool(r)) => Self::Bool(l != r),
            _ => Self::Err,
        }
    }

    pub fn not(expr: Self) -> Self {
        match expr {
            Self::Int(i) => Self::Int(!i),
            Self::Bool(b) => Self::Bool(!b),
            _ => Self::Err,
        }
    }

    pub fn index(lhs: Self, rhs: Self) -> Self {
        match lhs {
            Self::List(l) => {
                if let Self::Int(idx) = rhs
                    && idx >= 0
                {
                    match l.get(idx as usize) {
                        Some(e) => e.clone(),
                        None => Self::Err,
                    }
                } else {
                    Self::Err
                }
            }
            Self::Obj(o) => {
                let key = to_qobj_key(rhs);
                match o.get(&key) {
                    Some(v) => v.clone(),
                    None => Self::Err,
                }
            }
            _ => Self::Err,
        }
    }

    pub fn eq(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Bool(l == r),
            (Self::Float(l), Self::Float(r)) => Self::Bool(l == r),
            (Self::Bool(l), Self::Bool(r)) => Self::Bool(l == r),
            (Self::Str(l), Self::Str(r)) => Self::Bool(l == r),
            (Self::Void, Self::Void) => Self::Bool(true),
            (Self::Err, Self::Err) => Self::Bool(true),
            (Self::List(l), Self::List(r)) => {
                if l.len() == r.len() {
                    for (first, second) in l.into_iter().zip(r) {
                        if let Self::Bool(false) = Self::eq(first, second) {
                            return Self::Bool(false);
                        } else {
                        }
                    }
                    Self::Bool(true)
                } else {
                    Self::Bool(false)
                }
            }
            (Self::Obj(l), Self::Obj(r)) => todo!(),
            (Self::Thread(l), Self::Thread(r)) => match (l, r) {
                (Some(first), Some(second)) => Self::Bool(first == second),
                (None, None) => Self::Bool(true),
                (Some(defined), None) | (None, Some(defined)) => todo!(
                    "Define some mechanism to determine if a defined global thread number is the current thread"
                ),
            },
            (Self::Func(_, _), Self::Func(_, _)) => Self::Bool(false),
            _ => Self::Bool(false),
        }
    }

    pub fn ne(lhs: Self, rhs: Self) -> Self {
        match Self::eq(lhs, rhs) {
            Self::Bool(b) => Self::Bool(!b),
            _ => Self::Err,
        }
    }

    pub fn lt(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Bool(l < r),
            (Self::Float(l), Self::Float(r)) => Self::Bool(l < r),
            (Self::Str(l), Self::Str(r)) => Self::Bool(l < r),
            (Self::Thread(None), Self::Thread(_)) | (Self::Thread(_), Self::Thread(None)) => {
                Self::Bool(false)
            }
            (Self::Thread(Some(l)), Self::Thread(Some(r))) => Self::Bool(l < r),
            _ => Self::Bool(false),
        }
    }

    pub fn gt(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Bool(l > r),
            (Self::Float(l), Self::Float(r)) => Self::Bool(l > r),
            (Self::Str(l), Self::Str(r)) => Self::Bool(l > r),
            (Self::Thread(None), Self::Thread(_)) | (Self::Thread(_), Self::Thread(None)) => {
                Self::Bool(false)
            }
            (Self::Thread(Some(l)), Self::Thread(Some(r))) => Self::Bool(l > r),
            _ => Self::Bool(false),
        }
    }

    pub fn le(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Bool(l <= r),
            (Self::Float(l), Self::Float(r)) => Self::Bool(l <= r),
            (Self::Str(l), Self::Str(r)) => Self::Bool(l <= r),
            (Self::Thread(None), Self::Thread(_)) | (Self::Thread(_), Self::Thread(None)) => {
                Self::Bool(false)
            }
            (Self::Thread(Some(l)), Self::Thread(Some(r))) => Self::Bool(l <= r),
            _ => Self::Bool(false),
        }
    }

    pub fn ge(lhs: Self, rhs: Self) -> Self {
        match (lhs, rhs) {
            (Self::Int(l), Self::Int(r)) => Self::Bool(l >= r),
            (Self::Float(l), Self::Float(r)) => Self::Bool(l >= r),
            (Self::Str(l), Self::Str(r)) => Self::Bool(l >= r),
            (Self::Thread(None), Self::Thread(_)) | (Self::Thread(_), Self::Thread(None)) => {
                Self::Bool(false)
            }
            (Self::Thread(Some(l)), Self::Thread(Some(r))) => Self::Bool(l >= r),
            _ => Self::Bool(false),
        }
    }
}

fn from_qobj_key(k: String) -> QType {
    if let Some('$') = k.chars().nth(0) {
        QType::Str(k[1..].to_string())
    } else {
        QType::Int(k.parse::<i64>().unwrap())
    }
}

fn to_qobj_key(k: QType) -> String {
    match k {
        QType::Int(i) => format!("{i}"),
        QType::Str(s) => format!("${s}"),
        _ => panic!("How are you here?"),
    }
}
