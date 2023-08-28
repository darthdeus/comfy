use crate::*;
use mlua::{MetaMethod, UserData, UserDataMethods};

macro_rules! lua_method_1 {
    ($methods:expr, $name:ident) => {
        $methods
            .add_method(stringify!($name), |_, this, val| Ok(this.$name(val)));
    };
}

macro_rules! lua_method_2 {
    ($methods:expr, $name:ident) => {
        $methods.add_method(stringify!($name), |_, this, (v1, v2)| {
            Ok(this.$name(v1, v2))
        });
    };
}

#[macro_export]
macro_rules! lua_static_method_macros {
    ($globals:expr, $lua:expr, $type:ident, $wrapper:ident, $static_prefix:literal) => {
        macro_rules! lua_static_method {
            // no parameters
            ($name:ident, Pass) => {
                $globals.set(
                    concat!($static_prefix, stringify!($name)),
                    $lua.create_function(|_, ()| Ok($type::$name()))?,
                )?;
            };
            ($name:ident, Wrap) => {
                $globals.set(
                    concat!($static_prefix, stringify!($name)),
                    $lua.create_function(|_, ()| Ok($wrapper($type::$name())))?,
                )?;
            };

            // one parameter
            ($name:ident, Pass, Pass) => {
                $globals.set(
                    concat!($static_prefix, stringify!($name)),
                    $lua.create_function(|_, p1| Ok($type::$name(p1)))?,
                )?;
            };
            ($name:ident, Pass, Wrap) => {
                $globals.set(
                    concat!($static_prefix, stringify!($name)),
                    $lua.create_function(|_, p1| {
                        Ok($wrapper($type::$name(p1)))
                    })?,
                )?;
            };
            ($name:ident, Wrap, Pass) => {
                $globals.set(
                    concat!($static_prefix, stringify!($name)),
                    $lua.create_function(|_, p1: $wrapper| {
                        Ok($type::$name(p1.into()))
                    })?,
                )?;
            };
            ($name:ident, Wrap, Wrap) => {
                $globals.set(
                    concat!($static_prefix, stringify!($name)),
                    $lua.create_function(|_, p1: $wrapper| {
                        Ok($wrapper($type::$name(p1.into())))
                    })?,
                )?;
            };
        }
    };
}

macro_rules! lua_method_macros {
    ($methods:expr, $type:ident, $wrapper:ident) => {
        macro_rules! lua_method {
            // no parameters
            ($name:ident, Pass) => {
                $methods.add_method(stringify!($name), |_, data, ()| {
                    Ok(data.0.$name())
                });
            };
            ($name:ident, Wrap) => {
                $methods.add_method(stringify!($name), |_, data, ()| {
                    Ok($wrapper(data.0.$name()))
                });
            };

            // one parameter
            ($name:ident, Pass, Pass) => {
                $methods.add_method(stringify!($name), |_, data, p1| {
                    Ok(data.0.$name(p1))
                });
            };
            ($name:ident, Wrap, Pass) => {
                $methods
                    .add_method(stringify!($name), |_, data, p1: $wrapper| {
                        Ok(data.0.$name(p1.0))
                    });
            };
            ($name:ident, Pass, Wrap) => {
                $methods.add_method(stringify!($name), |_, data, p1| {
                    Ok($wrapper(data.0.$name(p1)))
                });
            };
            ($name:ident, Wrap, Wrap) => {
                $methods
                    .add_method(stringify!($name), |_, data, p1: $wrapper| {
                        Ok($wrapper(data.0.$name(p1.0)))
                    });
            };

            // two params
            ($name:ident, Pass, Pass, Pass) => {
                $methods.add_method(stringify!($name), |_, data, (p1, p2)| {
                    Ok(data.0.$name(p1, p2))
                });
            };
            ($name:ident, Wrap, Pass, Pass) => {
                $methods.add_method(
                    stringify!($name),
                    |_, data, (p1, p2): ($wrapper, _)| {
                        Ok(data.0.$name(p1.into(), p2))
                    },
                );
            };
            ($name:ident, Pass, Wrap, Pass) => {
                $methods.add_method(
                    stringify!($name),
                    |_, data, (p1, p2): (_, $wrapper)| {
                        Ok(data.0.$name(p1, p2.into()))
                    },
                );
            };
            ($name:ident, Pass, Pass, Wrap) => {
                $methods.add_method(stringify!($name), |_, data, (p1, p2)| {
                    Ok($wrapper(data.0.$name(p1, p2)))
                });
            };
            ($name:ident, Wrap, Wrap, Pass) => {
                $methods.add_method(
                    stringify!($name),
                    |_, data, (p1, p2): ($wrapper, $wrapper)| {
                        Ok(data.0.$name(p1.into(), p2.into()))
                    },
                );
            };
            ($name:ident, Wrap, Pass, Wrap) => {
                $methods.add_method(
                    stringify!($name),
                    |_, data, (p1, p2): ($wrapper, _)| {
                        Ok($wrapper(data.0.$name(p1.into(), p2)))
                    },
                );
            };
            ($name:ident, Pass, Wrap, Wrap) => {
                $methods.add_method(
                    stringify!($name),
                    |_, data, (p1, p2): (_, $wrapper)| {
                        Ok($wrapper(data.0.$name(p1, p2.into())))
                    },
                );
            };
            ($name:ident, Wrap, Wrap, Wrap) => {
                $methods.add_method(
                    stringify!($name),
                    |_, data, (p1, p2): ($wrapper, $wrapper)| {
                        Ok($wrapper(data.0.$name(p1.into(), p2.into())))
                    },
                );
            };
        }
    };
}

#[macro_export]
macro_rules! lua_field {
    ($fields:expr, $name:ident) => {
        $fields
            .add_field_method_get(stringify!($name), |_, data| Ok(data.$name));
        $fields.add_field_method_set(stringify!($name), |_, data, val| {
            data.$name = val;
            Ok(())
        });
    };
    ($fields:expr, $name:ident, Clone) => {
        $fields.add_field_method_get(stringify!($name), |_, data| {
            Ok(data.$name.clone())
        });
        $fields.add_field_method_set(stringify!($name), |_, data, val| {
            data.$name = val;
            Ok(())
        });
    };
    ($fields:expr, $name:ident, Wrap) => {
        $fields.add_field_method_get(stringify!($name), |_, data| {
            Ok(data.0.$name)
        });
        $fields.add_field_method_set(stringify!($name), |_, data, val| {
            data.0.$name = val;
            Ok(())
        });
    };
    ($fields:expr, $name:ident, $wrapper:ident) => {
        $fields.add_field_method_get(stringify!($name), |_, data| {
            Ok($wrapper(data.$name))
        });
        $fields.add_field_method_set(
            stringify!($name),
            |_, data, val: $wrapper| {
                data.$name = val.0;
                Ok(())
            },
        );
    };
}

#[derive(Copy, Clone, Debug)]
pub struct LuaIVec2(pub IVec2);

impl UserData for LuaIVec2 {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        lua_field!(fields, x, Wrap);
        lua_field!(fields, y, Wrap);
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(_methods: &mut M) {}
}

#[derive(Copy, Clone, Debug)]
pub struct LuaVec2(pub Vec2);

impl From<Vec2> for LuaVec2 {
    fn from(value: Vec2) -> Self {
        Self(value)
    }
}

impl From<LuaVec2> for Vec2 {
    fn from(val: LuaVec2) -> Self {
        val.0
    }
}

impl UserData for LuaVec2 {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        lua_field!(fields, x, Wrap);
        lua_field!(fields, y, Wrap);
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // lua_wrapped_method_0!(methods, length);

        methods.add_meta_function(
            MetaMethod::Add,
            |_, (a, b): (LuaVec2, LuaVec2)| Ok(LuaVec2(a.0 + b.0)),
        );

        methods.add_meta_function(
            MetaMethod::Sub,
            |_, (a, b): (LuaVec2, LuaVec2)| Ok(LuaVec2(a.0 - b.0)),
        );

        methods.add_meta_function(
            MetaMethod::Mul,
            |_, (a, b): (LuaVec2, LuaVec2)| Ok(LuaVec2(a.0 * b.0)),
        );

        methods.add_meta_function(
            MetaMethod::Div,
            |_, (a, b): (LuaVec2, LuaVec2)| Ok(LuaVec2(a.0 / b.0)),
        );

        methods.add_function("scale", |_, (a, b): (LuaVec2, f32)| {
            Ok(LuaVec2(a.0 * b))
        });

        methods.add_function("y_flip_scale", |_, (a, b): (LuaVec2, f32)| {
            Ok(LuaVec2(vec2(a.0.x * b, -a.0.y * b)))
        });

        lua_method_macros!(methods, Vec2, LuaVec2);

        lua_method!(abs, Wrap);
        lua_method!(abs_diff_eq, Wrap, Pass, Pass);
        lua_method!(angle_between, Wrap, Pass);
        // lua_method!(as_dvec2, Pass);
        // lua_method!(as_i64vec2, Pass);
        // lua_method!(as_ivec2, Pass);
        // lua_method!(as_u64vec2, Pass);
        // lua_method!(as_uvec2, Pass);
        lua_method!(ceil, Wrap);
        lua_method!(clamp, Wrap, Wrap, Wrap);
        lua_method!(clamp_length, Pass, Pass, Wrap);
        lua_method!(clamp_length_max, Pass, Wrap);
        lua_method!(clamp_length_min, Pass, Wrap);
        // lua_method!(cmpeq, Pass);
        // lua_method!(cmpge, Pass);
        // lua_method!(cmpgt, Pass);
        // lua_method!(cmple, Pass);
        // lua_method!(cmplt, Pass);
        // lua_method!(cmpne, Pass);
        lua_method!(copysign, Wrap, Wrap);
        lua_method!(distance, Wrap, Pass);
        lua_method!(distance_squared, Wrap, Pass);
        lua_method!(dot, Wrap, Pass);
        lua_method!(dot_into_vec, Wrap, Wrap);
        lua_method!(exp, Wrap);
        // lua_method!(extend, Pass, Wrap);
        lua_method!(floor, Wrap);
        lua_method!(fract, Wrap);
        lua_method!(is_finite, Pass);
        lua_method!(is_nan, Pass);
        // lua_method!(is_nan_mask, Pass);
        // lua_method!(is_negative_bitmask, Pass);
        lua_method!(is_normalized, Pass);
        lua_method!(length, Pass);
        lua_method!(length_recip, Pass);
        lua_method!(length_squared, Pass);
        lua_method!(lerp, Wrap, Pass, Wrap);
        lua_method!(max, Wrap, Wrap);
        lua_method!(max_element, Pass);
        lua_method!(min, Wrap, Wrap);
        lua_method!(min_element, Pass);
        // lua_method!(mul_add, Pass);

        lua_method!(normalize, Wrap);
        lua_method!(normalize_or_zero, Wrap);
        lua_method!(normalize_or_right, Wrap);
        lua_method!(perp, Wrap);
        lua_method!(perp_dot, Wrap, Pass);
        lua_method!(powf, Pass, Wrap);
        lua_method!(project_onto, Wrap, Wrap);
        lua_method!(project_onto_normalized, Wrap, Wrap);
        lua_method!(recip, Wrap);
        // lua_method!(reject_from, Pass);
        // lua_method!(reject_from_normalized, Pass);
        lua_method!(rotate, Wrap, Wrap);
        lua_method!(round, Wrap);
        // lua_method!(select, Pass);
        lua_method!(signum, Wrap);
        // lua_method!(to_array, Pass);
        // lua_method!(try_normalize, Pass);
        // lua_method!(write_to_slice, Pass);
    }
}

impl UserData for Color {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        lua_method_1!(methods, alpha);
        lua_method_1!(methods, darken);
        lua_method_2!(methods, mix);
        lua_method_1!(methods, boost);
        //methods.add_method("alpha", |_, color, val| Ok(color.alpha(val)));
        //methods.add_method("darken", |_, color, val| Ok(color.darken(val)));
        //methods.add_method("mix", |_, color, (other, val)| {
        //    Ok(color.mix(other, val))
        //});
    }

    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        lua_field!(fields, r);
        lua_field!(fields, g);
        lua_field!(fields, b);
        lua_field!(fields, a);
    }
}
