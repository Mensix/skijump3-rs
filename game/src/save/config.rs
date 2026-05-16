#[derive(Debug, Clone)]
pub struct Config {
    pub reg: i32,
    pub comphrs: i32,
    pub lct: i32,
    pub diff: i32,
    pub compactlist: i32,
    pub invback: i32,
    pub automatichrr: i32,
    pub beeppi: i32,
    pub nosamename: i32,
    pub goals: i32,
    pub diffwc: i32,
    pub kosystem: i32,
    pub languagenumber: i32,
    pub trainrounds: i32,
    pub namenumber: i32,
    pub setfile: String,
    pub gdetail: i32,
    pub seecomps: i32,
    pub jmaara: i32,
    pub jnimet: Vec<String>,
    pub pmaara: i32,
    pub profileorder: Vec<i32>,
    pub kothwind: i32,
    pub kothrounds: i32,
    pub kothpack: i32,
    pub kothmaki: i32,
    pub kothmaara: i32,
    pub kothpel: Vec<i32>,
    pub key_up: i32,
    pub key_right: i32,
    pub key_left: i32,
    pub key_telemark: i32,
    pub key_replay: i32,
    pub windplace: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reg: 1,
            comphrs: 1,
            lct: 0,
            diff: 0,
            compactlist: 0,
            invback: 0,
            automatichrr: 0,
            beeppi: 0,
            nosamename: 0,
            goals: 0,
            diffwc: 0,
            kosystem: 1,
            languagenumber: 255,
            trainrounds: 0,
            namenumber: 0,
            setfile: String::from("TEMP"),
            gdetail: 0,
            seecomps: 240,
            jmaara: 1,
            jnimet: vec![String::from("A TEAM")],
            pmaara: 1,
            profileorder: vec![1],
            kothwind: 0,
            kothrounds: 2,
            kothpack: 1,
            kothmaki: 0,
            kothmaara: 1,
            kothpel: vec![1],
            key_up: 72,
            key_right: 77,
            key_left: 75,
            key_telemark: 21524,
            key_replay: 21011,
            windplace: 1,
        }
    }
}

fn parse_num(s: &str) -> Result<i32, String> {
    s.trim().parse().map_err(|e| format!("Bad number '{s}': {e}"))
}

fn read_line<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, String> {
    lines.next().ok_or_else(|| "Unexpected end of CONFIG.SKI".to_string())
}

impl Config {
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        let s = std::str::from_utf8(data).map_err(|e| format!("Invalid UTF-8 in CONFIG.SKI: {e}"))?;
        let mut lines = s.lines();

        let reg = parse_num(read_line(&mut lines)?)?;
        let comphrs = parse_num(read_line(&mut lines)?)?;
        let lct = parse_num(read_line(&mut lines)?)?;
        let diff = parse_num(read_line(&mut lines)?)?;
        let compactlist = parse_num(read_line(&mut lines)?)?;
        let invback = parse_num(read_line(&mut lines)?)?;
        let automatichrr = parse_num(read_line(&mut lines)?)?;
        let beeppi = parse_num(read_line(&mut lines)?)?;
        let nosamename = parse_num(read_line(&mut lines)?)?;
        let goals = parse_num(read_line(&mut lines)?)?;
        let diffwc = parse_num(read_line(&mut lines)?)?;
        let kosystem = parse_num(read_line(&mut lines)?)?;
        let languagenumber = parse_num(read_line(&mut lines)?)?;
        let trainrounds = parse_num(read_line(&mut lines)?)?;
        let namenumber = parse_num(read_line(&mut lines)?)?;
        let setfile = read_line(&mut lines)?.trim().to_string();
        let gdetail = parse_num(read_line(&mut lines)?)?;
        let seecomps = parse_num(read_line(&mut lines)?)?;
        // 3 placeholder lines (19-21) originally BackGrSound/SBSounds/SoundBankNumber
        let _ = read_line(&mut lines)?;
        let _ = read_line(&mut lines)?;
        let _ = read_line(&mut lines)?;
        let jmaara = parse_num(read_line(&mut lines)?)?;

        let mut jnimet = Vec::new();
        for _ in 0..jmaara {
            jnimet.push(read_line(&mut lines)?.trim().to_string());
        }

        let pmaara = parse_num(read_line(&mut lines)?)?;

        let mut profileorder = Vec::new();
        for _ in 0..pmaara {
            profileorder.push(parse_num(read_line(&mut lines)?)?);
        }

        let kothwind = parse_num(read_line(&mut lines)?)?;
        let kothrounds = parse_num(read_line(&mut lines)?)?;
        let kothpack = parse_num(read_line(&mut lines)?)?;
        let kothmaki = parse_num(read_line(&mut lines)?)?;
        let kothmaara = parse_num(read_line(&mut lines)?)?;

        let mut kothpel = Vec::new();
        for _ in 0..kothmaara {
            kothpel.push(parse_num(read_line(&mut lines)?)?);
        }

        let key_up = parse_num(read_line(&mut lines)?)?;
        let key_right = parse_num(read_line(&mut lines)?)?;
        let key_left = parse_num(read_line(&mut lines)?)?;
        let key_telemark = parse_num(read_line(&mut lines)?)?;
        let key_replay = parse_num(read_line(&mut lines)?)?;
        let windplace = parse_num(read_line(&mut lines)?)?;
        // 2 placeholder lines (38-39) originally BaseIO/IRQ/DMA
        let _ = read_line(&mut lines)?;
        let _ = read_line(&mut lines)?;

        Ok(Self {
            reg, comphrs, lct, diff, compactlist, invback,
            automatichrr, beeppi, nosamename, goals, diffwc, kosystem,
            languagenumber, trainrounds, namenumber, setfile, gdetail, seecomps,
            jmaara, jnimet,
            pmaara, profileorder,
            kothwind, kothrounds, kothpack, kothmaki, kothmaara, kothpel,
            key_up, key_right, key_left, key_telemark, key_replay,
            windplace,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        macro_rules! num {
            ($val:expr) => {{
                let mut s = $val.to_string();
                s.push('\n');
                buf.extend_from_slice(s.as_bytes());
            }};
        }
        macro_rules! s_val {
            ($val:expr) => {{
                buf.extend_from_slice($val.as_bytes());
                buf.push(b'\n');
            }};
        }
        num!(self.reg);
        num!(self.comphrs);
        num!(self.lct);
        num!(self.diff);
        num!(self.compactlist);
        num!(self.invback);
        num!(self.automatichrr);
        num!(self.beeppi);
        num!(self.nosamename);
        num!(self.goals);
        num!(self.diffwc);
        num!(self.kosystem);
        num!(self.languagenumber);
        num!(self.trainrounds);
        num!(self.namenumber);
        s_val!(self.setfile);
        num!(self.gdetail);
        num!(self.seecomps);
        // 3 placeholder lines
        buf.extend_from_slice(b"0\n0\n0\n");
        num!(self.jmaara);
        for name in &self.jnimet {
            s_val!(name);
        }
        num!(self.pmaara);
        for idx in &self.profileorder {
            num!(idx);
        }
        num!(self.kothwind);
        num!(self.kothrounds);
        num!(self.kothpack);
        num!(self.kothmaki);
        num!(self.kothmaara);
        for idx in &self.kothpel {
            num!(idx);
        }
        num!(self.key_up);
        num!(self.key_right);
        num!(self.key_left);
        num!(self.key_telemark);
        num!(self.key_replay);
        num!(self.windplace);
        // 2 placeholder lines
        buf.extend_from_slice(b"0\n0\n");
        buf
    }
}

impl super::SaveFormat for Config {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}
