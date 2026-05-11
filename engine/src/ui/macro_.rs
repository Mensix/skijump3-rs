#[macro_export]
macro_rules! ui {
    () => { Vec::new() };

    (text($text:expr, $x:expr, $y:expr), $($rest:tt)*) => {{
        let mut v = ui!($($rest)*);
        v.push($crate::ui::Element::text($text, $x, $y));
        v
    }};

    (text_color($text:expr, $x:expr, $y:expr, $color:expr), $($rest:tt)*) => {{
        let mut v = ui!($($rest)*);
        v.push($crate::ui::Element::text_color($text, $x, $y, $color));
        v
    }};

    (sprite($idx:expr, $x:expr, $y:expr), $($rest:tt)*) => {{
        let mut v = ui!($($rest)*);
        v.push($crate::ui::Element::sprite($idx, $x, $y));
        v
    }};

    (fillbox($x:expr, $y:expr, $w:expr, $h:expr, $color:expr), $($rest:tt)*) => {{
        let mut v = ui!($($rest)*);
        v.push($crate::ui::Element::fillbox($x, $y, $w, $h, $color));
        v
    }};

    (box($x:expr, $y:expr, $w:expr, $h:expr, $color:expr), $($rest:tt)*) => {{
        let mut v = ui!($($rest)*);
        v.push($crate::ui::Element::box_($x, $y, $w, $h, $color));
        v
    }};

    (button($text:expr, $x:expr, $y:expr, $selected:expr, $cmd:expr), $($rest:tt)*) => {{
        let mut v = ui!($($rest)*);
        v.push($crate::ui::Element::button($text, $x, $y, $selected, $cmd));
        v
    }};

    ($elem:expr, $($rest:tt)*) => {{
        let mut v = ui!($($rest)*);
        v.insert(0, $elem);
        v
    }};
}