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
    pub jumper_count: i32,
    pub jnimet: Vec<String>,
    pub player_count: i32,
    pub profileorder: Vec<i32>,
    pub kothwind: i32,
    pub kothrounds: i32,
    pub kothpack: i32,
    pub kothmaki: i32,
    pub koth_count: i32,
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
            jumper_count: 1,
            jnimet: vec![String::from("A TEAM")],
            player_count: 1,
            profileorder: vec![1],
            kothwind: 0,
            kothrounds: 2,
            kothpack: 1,
            kothmaki: 0,
            koth_count: 1,
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
    s.trim()
        .parse()
        .map_err(|e| format!("Bad number '{s}': {e}"))
}

fn read_line<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, String> {
    lines
        .next()
        .ok_or_else(|| "Unexpected end of CONFIG.SKI".to_string())
}

impl Config {
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        let s =
            std::str::from_utf8(data).map_err(|e| format!("Invalid UTF-8 in CONFIG.SKI: {e}"))?;
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
        let jumper_count = parse_num(read_line(&mut lines)?)?;

        let mut jnimet = Vec::new();
        for _ in 0..jumper_count {
            jnimet.push(read_line(&mut lines)?.trim().to_string());
        }

        let player_count = parse_num(read_line(&mut lines)?)?;

        let mut profileorder = Vec::new();
        for _ in 0..player_count {
            profileorder.push(parse_num(read_line(&mut lines)?)?);
        }

        let kothwind = parse_num(read_line(&mut lines)?)?;
        let kothrounds = parse_num(read_line(&mut lines)?)?;
        let kothpack = parse_num(read_line(&mut lines)?)?;
        let kothmaki = parse_num(read_line(&mut lines)?)?;
        let koth_count = parse_num(read_line(&mut lines)?)?;

        let mut kothpel = Vec::new();
        for _ in 0..koth_count {
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
            reg,
            comphrs,
            lct,
            diff,
            compactlist,
            invback,
            automatichrr,
            beeppi,
            nosamename,
            goals,
            diffwc,
            kosystem,
            languagenumber,
            trainrounds,
            namenumber,
            setfile,
            gdetail,
            seecomps,
            jumper_count,
            jnimet,
            player_count,
            profileorder,
            kothwind,
            kothrounds,
            kothpack,
            kothmaki,
            koth_count,
            kothpel,
            key_up,
            key_right,
            key_left,
            key_telemark,
            key_replay,
            windplace,
        })
    }

    fn write_num(buf: &mut Vec<u8>, val: i32) {
        let mut s = val.to_string();
        s.push('\n');
        buf.extend_from_slice(s.as_bytes());
    }

    fn write_str(buf: &mut Vec<u8>, val: &str) {
        buf.extend_from_slice(val.as_bytes());
        buf.push(b'\n');
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        Self::write_num(&mut buf, self.reg);
        Self::write_num(&mut buf, self.comphrs);
        Self::write_num(&mut buf, self.lct);
        Self::write_num(&mut buf, self.diff);
        Self::write_num(&mut buf, self.compactlist);
        Self::write_num(&mut buf, self.invback);
        Self::write_num(&mut buf, self.automatichrr);
        Self::write_num(&mut buf, self.beeppi);
        Self::write_num(&mut buf, self.nosamename);
        Self::write_num(&mut buf, self.goals);
        Self::write_num(&mut buf, self.diffwc);
        Self::write_num(&mut buf, self.kosystem);
        Self::write_num(&mut buf, self.languagenumber);
        Self::write_num(&mut buf, self.trainrounds);
        Self::write_num(&mut buf, self.namenumber);
        Self::write_str(&mut buf, &self.setfile);
        Self::write_num(&mut buf, self.gdetail);
        Self::write_num(&mut buf, self.seecomps);
        // 3 placeholder lines
        buf.extend_from_slice(b"0\n0\n0\n");
        Self::write_num(&mut buf, self.jumper_count);
        for name in &self.jnimet {
            Self::write_str(&mut buf, name);
        }
        Self::write_num(&mut buf, self.player_count);
        for idx in &self.profileorder {
            Self::write_num(&mut buf, *idx);
        }
        Self::write_num(&mut buf, self.kothwind);
        Self::write_num(&mut buf, self.kothrounds);
        Self::write_num(&mut buf, self.kothpack);
        Self::write_num(&mut buf, self.kothmaki);
        Self::write_num(&mut buf, self.koth_count);
        for idx in &self.kothpel {
            Self::write_num(&mut buf, *idx);
        }
        Self::write_num(&mut buf, self.key_up);
        Self::write_num(&mut buf, self.key_right);
        Self::write_num(&mut buf, self.key_left);
        Self::write_num(&mut buf, self.key_telemark);
        Self::write_num(&mut buf, self.key_replay);
        Self::write_num(&mut buf, self.windplace);
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
