use crate::{ArgCount, CommandArgs, CommandBuilder};

enum ArgDescKind<T: CommandArgs> {
    Flag(&'static dyn Fn(&mut T, bool) -> std::result::Result<(), T::Error>),
    String {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, &str) -> std::result::Result<(), T::Error>,
    },
    U8 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, u8) -> std::result::Result<(), T::Error>,
    },
    U16 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, u16) -> std::result::Result<(), T::Error>,
    },
    U32 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, u32) -> std::result::Result<(), T::Error>,
    },
    U64 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, u64) -> std::result::Result<(), T::Error>,
    },
    U128 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, u128) -> std::result::Result<(), T::Error>,
    },
    Usize {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, usize) -> std::result::Result<(), T::Error>,
    },
    I8 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, i8) -> std::result::Result<(), T::Error>,
    },
    I16 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, i16) -> std::result::Result<(), T::Error>,
    },
    I32 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, i32) -> std::result::Result<(), T::Error>,
    },
    I64 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, i64) -> std::result::Result<(), T::Error>,
    },
    I128 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, i128) -> std::result::Result<(), T::Error>,
    },
    Isize {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, isize) -> std::result::Result<(), T::Error>,
    },
    F32 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, f32) -> std::result::Result<(), T::Error>,
    },
    F64 {
        count: ArgCount,
        default_value: Option<&'static str>,
        f: &'static dyn Fn(&mut T, f64) -> std::result::Result<(), T::Error>,
    },
}

pub struct ArgDescriptor<T: CommandArgs> {
    long: &'static str,
    short: Option<char>,
    help: &'static str,
    desc: ArgDescKind<T>,
}

macro_rules! arg_descriptor_arg_type_fn {
 {$f:ident, $t:ty, $e:ident } => {
     impl<T: CommandArgs> ArgDescriptor<T> {
         pub const fn $f<F: Fn(&mut T, $t) -> std::result::Result<(), T::Error>>(
             long: &'static str,
             short: Option<char>,
             help: &'static str,
             count: ArgCount,
             default_value: Option<&'static str>,
             f: &'static F,
         ) -> Self {
             Self {
                 long,
                 short,
                 help,
                 desc: ArgDescKind::$e {
                     count,
                     default_value,
                     f,
                 },
             }
         }
    }
    }
}

arg_descriptor_arg_type_fn!(arg_string, &str, String);
arg_descriptor_arg_type_fn!(arg_u8, u8, U8);
arg_descriptor_arg_type_fn!(arg_u16, u16, U16);
arg_descriptor_arg_type_fn!(arg_u32, u32, U32);
arg_descriptor_arg_type_fn!(arg_u64, u64, U64);
arg_descriptor_arg_type_fn!(arg_u128, u128, U128);
arg_descriptor_arg_type_fn!(arg_usize, usize, Usize);
arg_descriptor_arg_type_fn!(arg_i8, i8, I8);
arg_descriptor_arg_type_fn!(arg_i16, i16, I16);
arg_descriptor_arg_type_fn!(arg_i32, i32, I32);
arg_descriptor_arg_type_fn!(arg_i64, i64, I64);
arg_descriptor_arg_type_fn!(arg_i128, i128, I128);
arg_descriptor_arg_type_fn!(arg_isize, isize, Isize);
arg_descriptor_arg_type_fn!(arg_f64, f64, F64);
arg_descriptor_arg_type_fn!(arg_f32, f32, F32);

impl<T: CommandArgs> ArgDescriptor<T> {
    pub const fn arg_flag<F: Fn(&mut T, bool) -> std::result::Result<(), T::Error>>(
        long: &'static str,
        short: Option<char>,
        help: &'static str,
        f: &'static F,
    ) -> Self {
        Self {
            long,
            short,
            help,
            desc: ArgDescKind::Flag(f),
        }
    }

    pub(crate) fn build(&self, build: &mut CommandBuilder<T>) {
        match self.desc {
            ArgDescKind::Flag(f) => {
                build.add_flag(self.long, self.short, self.help, f);
            }
            ArgDescKind::String {
                count,
                default_value,
                f,
            } => {
                build.add_arg_string(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::U8 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_u8(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::U16 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_u16(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::U32 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_u32(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::U64 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_u64(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::U128 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_u128(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::Usize {
                count,
                default_value,
                f,
            } => {
                build.add_arg_usize(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::I8 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_i8(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::I16 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_i16(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::I32 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_i32(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::I64 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_i64(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::I128 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_i128(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::Isize {
                count,
                default_value,
                f,
            } => {
                build.add_arg_isize(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::F32 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_f32(self.long, self.short, self.help, count, default_value, f);
            }
            ArgDescKind::F64 {
                count,
                default_value,
                f,
            } => {
                build.add_arg_f64(self.long, self.short, self.help, count, default_value, f);
            }
            _ => {
                todo!();
            }
        }
    }
}
