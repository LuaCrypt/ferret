use ferret_crypto::Prng;

pub(super) struct Symbols {
    pub(super) words: String,
    pub(super) constants: String,
    pub(super) mask: String,
    pub(super) decode_words: String,
    pub(super) pack_words: String,
    pub(super) decode_bytes: String,
    pub(super) run: String,
    pub(super) load_const: String,
    pub(super) preload_consts: String,
    pub(super) cache: String,
}

pub(super) fn symbols(seed: u64) -> Symbols {
    let mut rng = Prng::new(seed ^ 0x7379_6d73);
    Symbols {
        words: ident(&mut rng, "w"),
        constants: ident(&mut rng, "c"),
        mask: ident(&mut rng, "m"),
        decode_words: ident(&mut rng, "dw"),
        pack_words: ident(&mut rng, "pw"),
        decode_bytes: ident(&mut rng, "db"),
        run: ident(&mut rng, "run"),
        load_const: ident(&mut rng, "k"),
        preload_consts: ident(&mut rng, "pk"),
        cache: ident(&mut rng, "cache"),
    }
}

impl Symbols {
    pub(super) fn apply(&self, code: &mut String) {
        for (from, to) in self.replacements() {
            *code = code.replace(from, to);
        }
    }

    fn replacements(&self) -> [(&'static str, &str); 10] {
        [
            ("@W@", &self.words),
            ("@C@", &self.constants),
            ("@M@", &self.mask),
            ("@DWV@", &self.decode_words),
            ("@PW@", &self.pack_words),
            ("@DB@", &self.decode_bytes),
            ("@RUN@", &self.run),
            ("@K@", &self.load_const),
            ("@PK@", &self.preload_consts),
            ("@CACHE@", &self.cache),
        ]
    }
}

fn ident(rng: &mut Prng, prefix: &str) -> String {
    format!("_f_{prefix}_{:08x}", rng.next_u32())
}
