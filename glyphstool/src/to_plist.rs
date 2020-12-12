pub use plist_derive::ToPlist;

use crate::plist::Plist;

pub trait ToPlist {
    fn to_plist(self) -> Plist;
}

pub trait ToPlistOpt {
    fn to_plist(self) -> Option<Plist>;
}

impl ToPlist for String {
    fn to_plist(self) -> Plist {
        self.into()
    }
}

impl ToPlist for bool {
    fn to_plist(self) -> Plist {
        (self as i64).into()
    }
}

impl ToPlist for i64 {
    fn to_plist(self) -> Plist {
        self.into()
    }
}

impl ToPlist for f64 {
    fn to_plist(self) -> Plist {
        self.into()
    }
}

impl<T: ToPlist> ToPlist for Vec<T> {
    fn to_plist(self) -> Plist {
        let mut result = Vec::new();
        for element in self {
            result.push(ToPlist::to_plist(element));
        }
        result.into()
    }
}

impl<T: ToPlist> ToPlistOpt for T {
    fn to_plist(self) -> Option<Plist> {
        Some(ToPlist::to_plist(self))
    }
}

impl<T: ToPlist> ToPlistOpt for Option<T> {
    fn to_plist(self) -> Option<Plist> {
        self.map(ToPlist::to_plist)
    }
}
