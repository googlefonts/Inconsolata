pub use plist_derive::FromPlist;

use crate::plist::Plist;

pub trait FromPlist {
    // Consider using result type; just unwrap for now.
    fn from_plist(plist: Plist) -> Self;
}

pub trait FromPlistOpt {
    // Consider using result type; just unwrap for now.
    fn from_plist(plist: Option<Plist>) -> Self;
}

impl FromPlist for String {
    fn from_plist(plist: Plist) -> Self {
        plist.into_string()
    }
}

impl FromPlist for bool {
    fn from_plist(plist: Plist) -> Self {
        // TODO: maybe error or warn on values other than 0, 1
        plist.as_i64().expect("expected integer") != 0
    }
}

impl FromPlist for i64 {
    fn from_plist(plist: Plist) -> Self {
        plist.as_i64().expect("expected integer")
    }
}

impl FromPlist for f64 {
    fn from_plist(plist: Plist) -> Self {
        plist.as_f64().expect("expected float")
    }
}

impl<T: FromPlist> FromPlist for Vec<T> {
    fn from_plist(plist: Plist) -> Self {
        let mut result = Vec::new();
        for element in plist.into_vec() {
            result.push(FromPlist::from_plist(element));
        }
        result
    }
}

impl<T: FromPlist> FromPlistOpt for T {
    fn from_plist(plist: Option<Plist>) -> Self {
        FromPlist::from_plist(plist.unwrap())
    }
}

impl<T: FromPlist> FromPlistOpt for Option<T> {
    fn from_plist(plist: Option<Plist>) -> Self {
        plist.map(FromPlist::from_plist)
    }
}
