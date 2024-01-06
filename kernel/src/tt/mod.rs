pub mod descriptor;
pub mod page;
pub mod table;

pub trait IntermediateLevel {
    type Next;
}

pub trait FinalLevel {}

macro_rules! define_levels {
    ($level:ident, $next_level:ident$(, $rest:ident)*) => {
        #[derive(Debug)]
        pub struct $level;

        impl IntermediateLevel for $level {
            type Next = $next_level;
        }

        define_levels!($next_level$(, $rest)*);
    };

    ($level:ident) => {
        #[derive(Debug)]
        pub struct $level;

        impl FinalLevel for $level {}
    };
}

define_levels!(Level0, Level1, Level2, Level3);
