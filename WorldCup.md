# World Cup Mode — Developer Guide

## Table of Contents

1. [Entry Points & Menu Flow](#1-entry-points--menu-flow)
2. [Competition State Machine](#2-competition-state-machine)
3. [UI Screens](#3-ui-screens)
4. [In-Jump UI Overlays](#4-in-jump-ui-overlays)
5. [KO System (Four Hills)](#5-ko-system-four-hills)
6. [Points & Scoring](#6-points--scoring)
7. [Qualification & Pre-Qualification](#7-qualification--pre-qualification)
8. [Injury System](#8-injury-system)
9. [Inherited Jump UI (shared with training)](#9-inherited-jump-ui-shared-with-training)
10. [Complete Code Map](#10-code-map)

---

## 1. Entry Points & Menu Flow

```
MainMenu (index 1) → JumpMenu (6 options, 7 items with exit)
                      ├─ 1. World Cup        → build_competition(CupStyle::WorldCup) → store.competition.start(comp) → RouteTarget::CompetitionJump
                      ├─ 2. Custom Cup       → RouteTarget::MainMenu (not implemented)
                      ├─ 3. Four Hills       → RouteTarget::MainMenu (not implemented)
                      ├─ 4. Team Cup         → RouteTarget::MainMenu (not implemented)
                      ├─ 5. Season Complete  → RouteTarget::MainMenu (not implemented)
                      ├─ 6. Practice         → RouteTarget::Practice
                      └─ 7. Exit             → RouteTarget::MainMenu
```

### Pascal references

| Pascal source | Line | What |
|---|---|---|
| `SJ3.PAS` | 5648–5720 | MainMenu procedure |
| `SJ3.PAS` | 5617–5645 | JumpMenu procedure |
| `SJ3.PAS` | 5257–5579 | `cup` procedure — full World Cup season loop |
| `SJ3UNIT.PAS` | 2884–2947 | Menu rendering engine (MakeMenu) |

### Known gap: Training rounds
Pascal's `cup` procedure uses `trainrounds` (default 2) for warm-up training rounds before qualification. The current Rust jump menu builds the competition with `trainrounds = 0`, skipping training. Pascal also stores `trainrounds` per profile and offers a settings toggle. The competition machine already supports `Training(n)` — the menu just needs to pass the profile's setting.

---

## 2. Competition State Machine

### Phase states (`CompetitionPhase` enum)

```
Setup ─→ Training(1) ─→ Training(2) ─→ … ─→ Training(N)
  │                                              │
  │                                    (if trainrounds > 0)
  │                                              │
  └──→ Qualification ─→ QualificationResults ─→ Round1 ─→ Round1Results ─→ Round2 ─→ Round2Results ─→ WorldCupStandings ─→ EventComplete ─→ (next hill) ─→ SeasonComplete
```

### Phase transition driver

`Competition::advance()` handles explicit transitions. `Competition::decide_next()` returns what the caller should do:

| Return value | Meaning |
|---|---|
| `Jump { idx, hill_idx, is_human }` | A participant must jump |
| `ShowResults` | Show results/standings screen, then caller calls `advance()` |
| `AdvancePhase` | Auto-advance through trivial phases (Training, Setup, EventComplete) |
| `Done` | Season is over |

### `world_cup_flow::drive()` loop

```
loop {
    match competition.decide_next() {
        Done → return WorldCupCommand::Done
        ShowResults → return WorldCupCommand::ShowResults
        AdvancePhase → competition.advance(); continue
        Jump { idx, hill_idx, is_human: true } → return HumanJump { participant, hill_idx, phase, is_new_event }
        Jump { idx, hill_idx, is_human: false } →
            simulate_computer(participant, hill_idx)
            competition.record_jump(outcome.score, outcome.distance)
            competition.advance()
            if competition.is_over() → return Done
            if next human found → return HumanJump { ... }
            // otherwise loop continues for more computers
    }
}
```

### Event setup sequence (Pascal `SJ3.PAS:5303-5314`)

```pascal
inc(osakilpailu);
nytmaki := HillOrder[osakilpailu];
Tuuli.Alusta(windplace);       // reset wind for new event
jumpalku;                       // load hill, reset scores per participant
```

Rust equivalence:

```rust
// In world_cup_jump.rs, detected by is_new_event in WorldCupCommand::HumanJump:
if is_new_event {
    JumpScene::setup_event(&store);  // store.jump_runtime.setup_event()
}
```

### Key variables (Pascal → Rust)

| Pascal | Rust | Scope |
|---|---|---|
| `wcup` (bool) | implicit via `competition.is_some()` | `Store.competition` |
| `cupstyle` (0=WC, 1=Custom, 2=4H) | `CupStyle` enum | `Competition.style` |
| `osakilpailu` | `Competition.current_event` (0-indexed) | `Competition` |
| `kierros` (-n..2) | `CompetitionPhase` | `Competition.phase` |
| `HillOrder[0..CupHills]` | `Competition.hill_order` | `Competition` |
| `nytmaki` | `Competition.current_hill()` | competition.current_hill() |
| `mcluett[0..NumPl]` | `Competition.field.master_order` | season points order |
| `luett[0..NumPl]` | `Competition.field.event_order` | event points order |
| `pisteet[player]` | `Participant.points` | per event score |
| `mcpisteet[player]` | `Participant.wc_points` | season WC points |
| `fourpts[player]` | `Participant.four_hills_points` | 4H/custom total |
| `qual[player]` | `Participant.qual` | `QualificationStatus` |
| `inj[player]` | `Participant.injury` | injury counter |
| `CStats[0..2,player]` | `Participant.{qual_len, round1_len, round2_len}` | jump lengths |
| `trainrounds` | `Competition.trainrounds` | | 

---

## 3. UI Screens

### Palette/Font color constants

| Name | Index | RGB |
|---|---|---|
| `FONT_DEFAULT` | 240 | white `[63,63,63]` |
| `FONT_HELP` | 241 | gray `[44,44,44]` |
| `FONT_GOLD` / `FONT_HEADER` | 246 | yellow `[63,63,21]` |
| `FONT_GREET` | 247 | turquoise `[9,57,63]` |
| `OTHER_NAME` | 241 (same as FONT_HELP) | gray |
| `OTHER_RANK` | 251 | (dynamic palette) |
| `OTHER_DISTANCE` | 252 | (dynamic palette) |
| `INJURY_COLOR` | 249 | (dynamic palette) |

### Screen background patterns

#### `new_screen(1)` — qualification/round results (SJ3GRAPH.PAS:99-103)

```
fillbox(0, 0, 319, 19, 245)     // top bar
fillbox(0, 20, 319, 199, 243)    // main area
FillArea(0, 0, 319, 199, 63)     // dither pattern
DrawAnim(5, 2, 62)               // logo sprite at (5,2)
MuutaMenu(1, color)              // tint palette based on `color` arg
```

`color` arg for `MuutaMenu`:
- `0` (quali): default
- `0` (round1): default
- `3` (WC standings): `MuutaMenu(1, 3)` → reddish tint
- `4` (4H): `MuutaMenu(1, 4)` → black tint
- `5` (stats): cyan tint
- `1` (team cup): neutral

Current Rust: `screen::new_screen(1)` followed by `apply_menu_tint(palette, 3, 0)` for results screens.

#### `new_screen(4)` — KO pairs screen (SJ3GRAPH.PAS:124-128)

```
fillbox(0, 0, 319, 19, 245)
fillbox(0, 20, 319, 119, 243)
fillbox(0, 120, 319, 139, 245)
fillbox(0, 140, 319, 199, 243)
DrawAnim(5, 2, 62)
DrawAnim(5, 122, 62)
```

### 3.1 — Jump Menu screen

```
Starts at MainMenu → item 1 → JumpMenu
Background: MainLayout
fillbox(1, 94, 116, 106, BG_LIST=8)
Header: lstr(18) at (11, 80), FONT_HEADER=246
7 menu items at y = 86 + temp*12 (temp=1..7), x=11
  "1 - World Cup"            lstr(27)
  "2 - Custom Cup"           lstrip(28)
  "3 - Four Hills"           lstrip(29)
  "4 - Team Cup"             lstrip(30)
  "5 - Season Complete"      lstrip(31)
  "6 - Practice"             lstrip(32)
  "7 - Exit"                 lstrip(33)  (y_off=12)
```

### 3.2 — Qualification Results screen

**Screen:** `new_screen(1)` with `MuutaMenu(1, 0)` → default palette

```
Header: lstr(82) + ' ' + osakilpailu + ' ' + lstr(8) + ' ' + CupHills + ' - ' + hill.name + ' K' + hill.kr
        at (30, 6), FONT_DEFAULT=240

Table columns at y=23, row step=7:
  Rank:   columnX=24  FONT_HEADER=246 (own) / OTHER_RANK=251 (other)
  Name:   columnX=32  FONT_DEFAULT=240 (own) / OTHER_NAME=241 (other)
  Points: columnX=184 FONT_DEFAULT=240 (own) / OTHER_NAME=241 (other)
  Length: columnX=199 FONT_GREET=247 (own) / OTHER_DISTANCE=252 (other)
  Qual:   columnX=252 FONT_HEADER=246 "Q" or FONT_GREET=247 "Q WC"
  Extra:  columnX=275 INJURY_COLOR=249 "INJ-N"

Qual status overlay:
  qual[who]=1 → "Q" at column 5
  qual[who]=2 (pre-qualified) → "Q WC" at column 5  (shown when sija>50)
  rank <= 30 (quali phase) → "Q" at column 5 (advancement indicator)
  injury > 0 → "INJ-(N-1)" at column 6

Distance format: "(length)" if single, "(len1-len2)" if two rounds
```

**Current Rust:** `results.rs` builds this via `build_results_page` + `render_results_page` with `new_screen(1)`.

### 3.3 — Round 1 Results screen

**Screen:** Same as qualification: `new_screen(1)`, `MuutaMenu(1, 0)`

```
Header: lstr(81) + ' ' + osakilpailu + ' ' + lstr(8) + ' ' + CupHills + ' - ' + hill.name + ' K' + hill.kr + ' - R 1'
        at (30, 6), FONT_DEFAULT=240

Same table layout as qualification.
Kierros-specific data:
  CStats[1,who] = round1_len
  participants with qual.len>0 (not DID_NOT_START) are listed
```

### 3.4 — Round 2 Results screen

**Screen:** Same as Round 1: `new_screen(1)`, `MuutaMenu(1, 0)`

```
Header: lstr(81) + ' ' + osakilpailu + ' ' + lstr(8) + ' ' + CupHills + ' - ' + hill.name + ' K' + hill.kr + ' - R 2'
        at (30, 6), FONT_DEFAULT=240

Two lengths shown: round1_len and round2_len
Only participants whose qual.can_jump() are listed
```

### 3.5 — World Cup Standings screen

**Screen:** `new_screen(1)` with `MuutaMenu(1, 3)` → reddish tint

```
Header cases:
  Final event: lstr(90) + ' ' + lstr(27 + cupstyle)   // e.g. "FINAL WORLD CUP STANDINGS"
  Mid-season:  lstr(27+cupstyle) + ' ' + lstr(87) + ' ' + txt(osakilpailu) + ' ' + lstr(8) + ' ' + txt(CupHills)
               e.g. "WORLD CUP STANDINGS EVENT 1/15"

Column points: mcpisteet[who] (WC points), not pisteet
Only participants with wc_points > 0 are shown
```

### 3.6 — 4 Hills / Custom standings screen

**Screen:** `new_screen(1)` with `MuutaMenu(1, 2)` (4H=black tint) or phase-dependent color

Header:
```pascal
// 4 Hills final:
lstr(85)  // "FOUR HILLS TOURNAMENT 2001"

// 4 Hills mid-season:
lstr(84) + ' ' + txt(temp3) + ' ' + lstr(83) + ' - ' + hill.name + ' K' + hill.kr
// e.g. "FOUR HILLS STANDINGS 2/4 - GROSS-TITLISBERG K185"
```

### 3.7 — KO Pairs screen (Four Hills only)

**Screen:** `new_screen(4)` with logo at top and middle

```
Header at (30, 6): lstr(94)
Pair columns: xx=145+(column*30), yy=17+(row*7)
Left column at x=40: ewritefont points
Right column at x=303: ewritefont points

Colors:
  Left side: FONT_HELP=241
  Right side: FONT_HELP=241
  qual[who]=1 (winner): FONT_HEADER=246 (via Muuta palette 251)
  qual[who]=2 (lucky loser): special color (252)
  own player: FONT_DEFAULT=240

Pair labels: writefont(154, yy, 'vs.')
```

### 3.8 — Season Complete screen

**Pascal behavior:** At `osakilpailu = CupHills`, after final event results and standings, the loop exits and returns to main menu. No special "season complete" screen in Pascal — the last standings screen effectively serves this purpose.

**Current Rust:** `RenderMode::Done` returns `vec![]` (empty). Should show final World Cup standings with appropriate "FINAL" header, then either auto-return to menu or wait for ESC.

### 3.9 — Results page navigation (Rust)

```rust
Event::Keyboard(Key::Right | Key::Char(' ')) → display_page++
Event::Keyboard(Key::Left)                   → display_page--
Event::Keyboard(Key::Escape | Key::Enter)    → display_page = 0; competition.advance()
```

Pages: `QUALIFICATION_ITEMS_PER_PAGE = 25` items per page.

### 3.10 — Compact sign (Pascal)

When `compactlist` is enabled, footer text at (30, 190):
```
lstr(86)   // "COMPACT"
```
Color: FONT_HELP=241.

---

## 4. In-Jump UI Overlays

During a World Cup jump, the info panel at x≈308 cycles through information.

### 4.1 — Hill info (drawtop5info)

Pascal `SJ3.PAS:896-922`:

```
(308, 9):  hill.name + ' K' + hill.kr
(308, 13+temp*7, temp=0..4):  name + '$' + points (top 5 current event)
```

Info cycle (Pascal `SJ3.PAS:965-996`):

| Frame range | Overlay |
|---|---|
| 0–130 | Hill record info (`drawhrinfo`) |
| 146–276 | WC standings top 5 (`drawwcinfo`) |
| 291 | Reset cycle |
| — | — |
| 0–130 | Top 5 event points (`drawtop5info`) |
| 146–276 | Hill record info (`drawhrinfo`) |
| 292–422 | WC standings top 5 if leader has >0 points (`drawwcinfo`) |
| 437 | Reset cycle |

### 4.2 — WC info panel (drawwcinfo)

Pascal `SJ3.PAS:927-952`:

```
(308, 9):  lstr(70) if World Cup, lstr(71) if Custom Cup  // "WC" / "CUSTOM CUP"
(308, 13+temp*7, temp=1..5):
    name + '$' + points (if mcpisteet > 0)
    if diffwc and temp>1: shows difference to leader: (mcpisteet[temp] - mcpisteet[1])
```

### 4.3 — Round 2 current position (Pascal `SJ3.PAS:916`)

```
(308, 62):  lstr(63) + ': ' + txtp(temp+1) at kierros=2, index=1, wcup=true
```

### 4.4 — Hill record info (drawhrinfo)

Shows current hill record distance and holder (shared with training mode).

---

## 5. KO System (Four Hills)

Only active when `kosystem` is true (Four Hills tournament).

### Qualification KO pair building

```pascal
for temp:=25 downto 1 do
  for temp2:=0 to 1 do
    index:=51-temp;   // upper half
    if (temp2=1) then index:=temp;  // lower half
    hyppy(index, luett[index], 0);
```

### Round 2 KO pair building

```pascal
for temp:=25 downto 1 do
  if pisteet[luett[temp]] >= pisteet[luett[51-temp]] then
    qual[luett[temp]]:=1       // winner
  else
    qual[luett[51-temp]]:=1;   // winner

// Top 5 lucky losers (after sorting by points)
for temp:=1 to 5 do
  find next unqualified participant
  qual[them]:=2;  // lucky loser
```

### KO status values

| Value | Meaning |
|---|---|
| `qual[x]=0` | eliminated |
| `qual[x]=1` | advanced (round winner) |
| `qual[x]=2` | lucky loser (top 5 non-winners by points) |

### Rust status notes

`QualificationStatus` already defines:
- `NotQualified`
- `Qualified`
- `Eliminated`
- `PreQualified`
- `LuckyLoser` (defined but unused)
- `KoSeed` (defined but unused)

KO pair screen UI renders brackets with two logo headers (top and middle sections), points for each competitor, `vs.` label, and color-coded status.

---

## 6. Points & Scoring

### World Cup points table (`SJ3UNIT.PAS:19-21`)

```pascal
WCpoints: array[1..30] of byte =
  (100, 80, 60, 50, 45, 40, 36, 32, 29, 26,
   24, 22, 20, 18, 16, 15, 14, 13, 12, 11,
   10,  9,  8,  7,  6,  5,  4,  3,  2,  1);
```

Only positions 1–30 receive points. Already implemented in `competition/scoring.rs`.

### Award timing (Pascal `SJ3.PAS:5542-5543`)

```pascal
for temp:=1 to NumPl do
  if (sija[temp] < 31) then
    inc(mcpisteet[temp], WCPoints[sija[temp]]);
```

Awarded after Round 2 results before showing WC standings screen.

### Custom Cup / Four Hills scoring

```pascal
// Four Hills (cupstyle=2, sortby=1)
for temp:=1 to NumPl do
  fourpts[temp] += pisteet[temp];
```

### Jump score recording (Pascal `SJ3.PAS:2239-2257`)

```pascal
if (not jcup) and (kierros >= 0) then begin
  inc(pisteet[pel], score);
  CStats[kierros, pel] := hp;
end;
if (kierros >= 0) then begin
  stats[statsvictim, osakilpailu].RoundPts[kierros] := score;
  stats[statsvictim, osakilpailu].RoundLen[kierros] := hp;
end;
```

Rust equivalence in `Competition::record_jump()`:

```rust
match self.phase {
    Training(_) | Qualification => {
        field.get_mut(idx).points = jump_points;
        field.get_mut(idx).qual_len = length;
    }
    Round1 => {
        field.get_mut(idx).points = jump_points;
        field.get_mut(idx).round1_len = length;
    }
    Round2 => {
        field.get_mut(idx).points += jump_points;
        field.get_mut(idx).round2_len = length;
    }
    _ => {}
}
```

Note: In Rust, Round2 adds to existing points from Round1 (matching Pascal's `inc(pisteet[pel], score)` which accumulates). The `pisteet` array is not reset between Round1 and Round2 — it's set to 0 once at the start of the event and then accumulates through all rounds including training/qualification.

---

## 7. Qualification & Pre-Qualification

### Pre-Qualification (Pascal `SJ3.PAS:5325-5330`)

```pascal
if (osakilpailu > 1) then
  for temp := 1 to NumPl do
    if (sija[temp] < 11) and (inj[temp] = 0) then begin
      qual[temp] := 2;
      inc(qual[0]);
    end;
```

- Only applies from event 2 onwards
- Top 10 in WC standings (sija < 11) are pre-qualified
- `qual[0]` counts pre-qualified jumpers (used to reduce qualification spots)

### Qualification list building

Pascal start list for qualification:
```pascal
// Reverse mcluett (WC standings order), non-injured, skip qual[temp]=2 (prequalified)
for index := NumPl downto 1 do
  if (inj[mcluett[index]] = 0) and (qual[mcluett[index]] != 2) then
    hyppy(index, mcluett[index], 0);
```

Rust already implements this in `CompetitionField::build_start_list`:

```rust
CompetitionPhase::Qualification => self
    .master_order
    .iter()
    .rev()
    .filter(|&&idx| self.participants[idx].injury == 0
        && self.participants[idx].qual != QualificationStatus::PreQualified)
    .copied()
    .collect(),
```

### Qualification selection (Pascal `SJ3.PAS:5379-5404`)

```pascal
// Sort by event points (pisteet) into luett
jarjestys(2, 1, NumPl);

// Top 50 minus pre-qualified count get qualified
qual[0] counts pre-qualified
for temp := 1 to 50 - qual[0] do
  find next unqualified, uninjured participant
  qual[temp] := 1;

// Ties are also qualified
for temp := 51 - qual[0] to NumPl do
  if (same rank as last qualified) and (injured=0) then
    qual[temp] := 1;

// KO system special case: first 50 all get qual=3
if (dokosystem) then
  for temp := 1 to 50 do qual[temp] := 3
else
  // Everyone with rank < 51 gets qual=1 (already covered above)
  for temp := 1 to NumPl do
    if (sija[temp] < 51) then qual[temp] := 1;
```

Rust implements this in `machine.rs:308-332`:

```rust
fn resolve_qualification(&mut self) {
    self.field.sort_field(SortBy::EventPoints);
    let pre_qualified = self.field.iter().filter(|p| p.qual == PreQualified).count();
    let spots = QUALIFICATION_SPOTS - pre_qualified;  // 50 - qual[0]
    let mut taken = 0usize;
    for idx in self.field.event_order.clone() {
        let p = self.field.get(idx);
        if p.injury > 0 || p.qual == PreQualified { continue; }
        if taken < spots {
            self.field.get_mut(idx).qual = Qualified;
            taken += 1;
        } else {
            self.field.get_mut(idx).qual = Eliminated;
        }
    }
}
```

### Qualification status indicators on results screen

| Status | Column 5 marker |
|---|---|
| `Qualified` (qual=1) | `"Q"` in FONT_HEADER (col2) |
| `PreQualified` (qual=2) | `"Q WC"` in FONT_GREET (col3), only shown when rank > 50 |
| `Eliminated` (qual=0) | nothing |
| Tie-breakers | Additional qualifiers at same rank |

### DID_NOT_START

```pascal
// All non-qualified participants get -5555 as pisteet
for temp := 1 to NumPl do
  if (qual[temp] = 0) then pisteet[temp] := -5555;
```

Rust constant: `DID_NOT_START_SCORE = -5555`.

---

## 8. Injury System

### Pascal (`SJ3.PAS:5318-5319`)

```pascal
for temp := 1 to NumPl do
  if (inj[temp] > 0) then dec(inj[temp]);
```

- Injuries are decremented by 1 at each event start
- Injured jumpers are skipped in start lists
- Injury display: `"INJ-(N-1)"` on results screen where N = `inj[player]`

### Rust

- `Participant.injury: u8` field exists
- `Competition::advance()` calls `tick_injuries()` during Setup
- `CompetitionField::tick_injuries()` decrements positive injuries
- Injured participants are filtered out of start lists

---

## 9. Inherited Jump UI (shared with training)

The jump view (visible jump scene) is fully shared between training and World Cup through `JumpScene` + `JumpRunner`. The UI elements rendered during flight/landing/result are identical.

### Jump info panel (x=308 side panel)

During World Cup, the jump info panel cycles through:

1. **Top 5 event points** (`drawtop5info`):
   - Line 1: hill name + K at (308, 9)
   - Lines 2–6: name + points at (308, 13+temp*7)

2. **WC standings** (`drawwcinfo`):
   - "WC" / "CUSTOM CUP" at (308, 9)
   - Top 5 WC points with diff-to-leader at (308, 13+temp*7)

3. **Hill record** (`drawhrinfo`):
   - Record distance and holder info

### Info cycle timing

In Pascal, the cycle runs on frame counter `l` (starts at 0 for each jump):

```
Cycle set 1:
  l=0..130:     hill record
  l=146..276:   WC standings
  l=291:        reset to 0

Cycle set 2:
  l=0..130:     top 5 event
  l=146..276:   hill record
  l=292..422:   WC standings (if leader has >0 points)
  l=437:        reset to 0
```

This is not yet implemented in Rust — currently shows static information.

---

## 10. Complete Code Map

### Entry point files

| File | Purpose |
|---|---|
| `game/src/route.rs` | `RouteTarget::CompetitionJump` enum variant |
| `game/src/lib.rs:198-202` | Router mapping to `WorldCupJumpView` |
| `game/src/views/menu/main.rs:13-21` | Main menu → JumpMenu |
| `game/src/views/menu/jump.rs:17-25` | Jump menu actions |
| `game/src/views/menu/jump.rs:103-117` | World Cup build + start |

### View layer

| File | Purpose |
|---|---|
| `game/src/views/jump/world_cup_jump.rs` | Main WC view — renders Jump/Results/Done |
| `game/src/views/jump/results.rs` | Results page builder and renderer |
| `game/src/views/jump/training_jump.rs` | Training jump (reference for UI patterns) |
| `game/src/views/jump/training_setup.rs` | Practice hill selection (reference) |
| `game/src/views/replay/playback.rs` | Replay mode info panel (reference) |

### Controller layer

| File | Purpose |
|---|---|
| `game/src/controllers/jump_scene.rs` | Wraps JumpRunner, handles events, snow, wind, rendering |
| `game/src/controllers/world_cup_flow.rs` | Pure drive() loop — converts Competition state to commands |
| `game/src/controllers/jump_input.rs` | JumpInputController (shared) |

### Competition model

| File | Purpose |
|---|---|
| `game/src/competition/types.rs` | `CompetitionPhase`, `CupStyle`, `Participant`, `QualificationStatus` |
| `game/src/competition/machine.rs` | `Competition` — state machine, decide_next, advance, record_jump |
| `game/src/competition/field.rs` | `CompetitionField` — participants, orders, start lists, sorting, ranking |
| `game/src/competition/builder.rs` | `build_competition` — creates Competition from profiles |
| `game/src/competition/scoring.rs` | `award_wc_points` — WC points table |

### Store

| File | Purpose |
|---|---|
| `game/src/store.rs:151-205` | `CompetitionSlot` — wraps `RefCell<Option<Competition>>` |

### Shared jump infrastructure

| File | Purpose |
|---|---|
| `game/src/jump/runner.rs` | `JumpRunner` — owns session, snow, physics, computer AI |
| `game/src/jump/session.rs` | `JumpSession` —   physics, replay, camera, phase state |
| `game/src/jump/presentation.rs` | Element builders for jump view (info, result, wind gauge, etc.) |
| `game/src/jump/config.rs` | `JumpConfig`, `JumpParticipant` |
| `game/src/jump/policy.rs` | `JumpPolicy`, `JumperControl` |
| `game/src/jump/types.rs` | `JumpPhase`, `FlightWind`, `JumpInput`, `JumpOutcome` |
| `game/src/jump/animation.rs` | Animation tables from Pascal |
| `game/src/jump/state.rs` | `JumpState` — physics state machine |
| `game/src/jump/scoring.rs` | Jump scoring (judges, distance points, etc.) |
| `game/src/jump/wind.rs` | Wind model |
| `game/src/jump/snow.rs` | Snow system |
| `game/src/jump/replay.rs` | Replay trace recording/reading |
| `game/src/jump/replay_player.rs` | Replay playback |

### UI components

| File | Purpose |
|---|---|
| `game/src/components/screen.rs` | `new_screen()`, `page_hints()` |
| `game/src/gfx/palette.rs` | Color constants, `apply_menu_tint()` |

---

## Implementation checklist

### Phase 1 — Core competition flow (done)

- [x] `Competition` state machine with all phase transitions
- [x] `world_cup_flow::drive()` loop
- [x] `WorldCupJumpView` scaffold (Jump / Results / Done modes)
- [x] `JumpScene` integration (visible jump, hidden simulation)
- [x] Basic results page rendering

### Phase 2 — UI parity (needs work)

- [ ] KO pair screen (Four Hills)
- [ ] In-jump info cycle (top5 event / WC standings / hill record)
- [ ] `SeasonComplete` → final standings with "FINAL" header, then exit
- [ ] Compact list toggle (compact sign)
- [ ] Proper `new_screen(1/4)` with correct `MuutaMenu` color parameter per phase
- [ ] Stats/detail screen (`showstats`)
- [ ] Round 2 current-position overlay "(308, 62)"

### Phase 3 — Features (needs work)

- [ ] Training rounds before qualification (read from profile)
- [ ] KO system for Four Hills
- [ ] Custom Cup (`cupstyle=1`) — hill selection screen
- [ ] Four Hills mode (`cupstyle=2`) — 4 fixed hills with KO
- [ ] Team Cup (`cupstyle=4`)
- [ ] Injury creation (fall risk in landing)
- [ ] Profile skipquali setting
- [ ] World Cup scoring diff display (positive/negative delta from leader)

### Phase 4 — Polish

- [ ] Season-end hall of fame / credtis
- [ ] `MakeSendMe` (high score submission) hook
- [ ] F10 / ESC abort mid-season
- [ ] Profile stats persistence per event
