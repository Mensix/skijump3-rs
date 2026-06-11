use crate::components::screen::new_screen_with_bg;
use crate::gfx::palette::{BG_TEAMCUP, BLACK, FONT_DEFAULT, FONT_GOLD, FONT_HELP};
use crate::jump::hud;
use crate::store::{ResourcesRef, StoreRef};
use crate::text::layout::shorten_name;
use engine::ui::Element;

pub(crate) fn team_names(store: &StoreRef) -> Vec<String> {
    store
        .with_active(|active| {
            let tc = active.team_cup_runtime()?;
            Some(
                tc.teams
                    .iter()
                    .filter(|t| t.is_human_team)
                    .map(|t| t.name.clone())
                    .collect(),
            )
        })
        .flatten()
        .unwrap_or_default()
}

pub(crate) fn team_x(team_idx: usize) -> i32 {
    if team_idx == 0 {
        30
    } else {
        160
    }
}

pub(crate) fn naming_elements(
    resources: &ResourcesRef,
    store: &StoreRef,
    team_names: &[String],
    current_team: usize,
    name_buffer: &str,
    cursor_visible: bool,
) -> Vec<Element> {
    let mut els = new_screen_with_bg(1, BG_TEAMCUP);
    push_team_cup_header(&mut els, resources, store);

    for n in 0..team_names.len() {
        let xx = team_x(n);
        let is_current = current_team == n;
        push_jumper_names(&mut els, store, n, xx);

        if is_current {
            els.push(Element::text(
                format!("{} {}:", resources.langbase.lstr(113), n + 1),
                xx,
                30,
                FONT_DEFAULT,
                false,
            ));
            els.push(Element::fillbox(xx - 2, 40, 125, 10, BLACK));
            els.push(Element::text(
                name_buffer.to_string(),
                xx,
                42,
                FONT_DEFAULT,
                false,
            ));
            if cursor_visible {
                let cw = resources.font.string_width(name_buffer) as i32;
                els.push(Element::fillbox(xx + cw, 48, 5, 1, FONT_DEFAULT));
            }
        } else {
            push_named_team(&mut els, resources, team_names, n, xx);
        }
    }

    els
}

pub(crate) fn ready_elements(
    resources: &ResourcesRef,
    store: &StoreRef,
    team_names: &[String],
    cursor_visible: bool,
) -> Vec<Element> {
    let mut els = new_screen_with_bg(1, BG_TEAMCUP);
    push_team_cup_header(&mut els, resources, store);

    for n in 0..team_names.len() {
        let xx = team_x(n);
        push_jumper_names(&mut els, store, n, xx);
        push_named_team(&mut els, resources, team_names, n, xx);
    }

    hud::push_wait_for_key(
        &mut els,
        &resources.langbase,
        305,
        180,
        BG_TEAMCUP,
        FONT_DEFAULT,
        FONT_DEFAULT,
        cursor_visible,
    );

    els
}

pub(crate) fn showteams_elements(
    resources: &ResourcesRef,
    store: &StoreRef,
    cursor_visible: bool,
) -> Vec<Element> {
    let mut els = new_screen_with_bg(1, BG_TEAMCUP);
    els.push(Element::text(
        resources.langbase.lstr(111).to_string(),
        30,
        6,
        FONT_DEFAULT,
        false,
    ));

    let mut x = 5i32;
    let mut y = 24i32;
    store.with_active(|active| {
        let Some(tc) = active.team_cup_runtime() else {
            return;
        };
        for &team_idx in tc.team_order.iter().rev() {
            let team = &tc.teams[team_idx];
            let is_human = team.is_human_team;

            els.push(Element::text(
                shorten_name(&team.name, &resources.font, 95),
                x,
                y,
                FONT_DEFAULT,
                false,
            ));

            let jcolor = if is_human { FONT_GOLD } else { FONT_HELP };
            for (j, member) in team.members.iter().enumerate() {
                els.push(Element::text(
                    shorten_name(&member.competitor.name, &resources.font, 90),
                    x + 4,
                    y + 7 + j as i32 * 6,
                    jcolor,
                    false,
                ));
            }

            x += 102;
            if x > 240 {
                x = 5;
                y += 35;
            }
        }
    });

    hud::push_wait_for_key(
        &mut els,
        &resources.langbase,
        305,
        6,
        BG_TEAMCUP,
        FONT_DEFAULT,
        FONT_DEFAULT,
        cursor_visible,
    );

    els
}

fn push_named_team(
    els: &mut Vec<Element>,
    resources: &ResourcesRef,
    team_names: &[String],
    n: usize,
    xx: i32,
) {
    els.push(Element::fillbox(xx - 10, 30, 135, 25, BG_TEAMCUP));
    els.push(Element::fill_area(63));
    els.push(Element::text(
        format!("{} {}:", resources.langbase.lstr(114), n + 1),
        xx,
        30,
        FONT_HELP,
        false,
    ));
    els.push(Element::text(
        team_names[n].clone(),
        xx,
        42,
        FONT_DEFAULT,
        false,
    ));
}

fn push_team_cup_header(els: &mut Vec<Element>, resources: &ResourcesRef, store: &StoreRef) {
    els.push(Element::text(
        resources.langbase.lstr(111).to_string(),
        30,
        6,
        FONT_DEFAULT,
        false,
    ));
    els.push(Element::text(
        resources.langbase.lstr(112).to_string(),
        30,
        110,
        FONT_DEFAULT,
        false,
    ));

    if let Some(schedule) = store
        .with_active(|active| active.team_cup_runtime().map(|tc| tc.schedule.clone()))
        .flatten()
    {
        for (i, &hill_idx) in schedule.iter().enumerate() {
            let hill_name = resources
                .hills
                .hill(hill_idx)
                .map(|h| format!("{}. {} K{}", i + 1, h.name, h.kr))
                .unwrap_or_else(|| format!("{}. Hill {}", i + 1, hill_idx));
            els.push(Element::text(
                hill_name,
                30,
                124 + i as i32 * 10,
                FONT_GOLD,
                false,
            ));
        }
    }
}

fn push_jumper_names(els: &mut Vec<Element>, store: &StoreRef, team_n: usize, xx: i32) {
    let jumpers: Vec<String> = store
        .with_active(|active| {
            let tc = active.team_cup_runtime()?;
            Some(
                tc.teams
                    .iter()
                    .filter(|t| t.is_human_team)
                    .nth(team_n)
                    .map(|t| {
                        t.members
                            .iter()
                            .map(|m| m.competitor.name.clone())
                            .collect()
                    })
                    .unwrap_or_default(),
            )
        })
        .flatten()
        .unwrap_or_default();
    for (j, jname) in jumpers.iter().enumerate() {
        els.push(Element::text(
            jname.clone(),
            xx + 13,
            56 + j as i32 * 10,
            FONT_GOLD,
            false,
        ));
    }
}
