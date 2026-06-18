use ferret_output::IdentGenerator;

pub(super) struct Symbols {
    pub(super) words: String,
    pub(super) constants: String,
    pub(super) mask: String,
    pub(super) decode_words: String,
    pub(super) pack_words: String,
    pub(super) decode_bytes: String,
    pub(super) pack_results: String,
    pub(super) run: String,
    pub(super) load_const: String,
    pub(super) preload_consts: String,
    pub(super) cache: String,
}

pub(super) fn symbols(seed: u64) -> Symbols {
    let mut idents = IdentGenerator::new(seed ^ 0x7379_6d73);
    Symbols {
        words: idents.ident(),
        constants: idents.ident(),
        mask: idents.ident(),
        decode_words: idents.ident(),
        pack_words: idents.ident(),
        decode_bytes: idents.ident(),
        pack_results: idents.ident(),
        run: idents.ident(),
        load_const: idents.ident(),
        preload_consts: idents.ident(),
        cache: idents.ident(),
    }
}

impl Symbols {
    pub(super) fn apply(&self, code: &mut String) {
        for (from, to) in self.replacements() {
            *code = code.replace(from, to);
        }
    }

    fn replacements(&self) -> [(&'static str, &str); 11] {
        [
            ("@W@", &self.words),
            ("@C@", &self.constants),
            ("@M@", &self.mask),
            ("@DWV@", &self.decode_words),
            ("@PW@", &self.pack_words),
            ("@DB@", &self.decode_bytes),
            ("@PR@", &self.pack_results),
            ("@RUN@", &self.run),
            ("@K@", &self.load_const),
            ("@PK@", &self.preload_consts),
            ("@CACHE@", &self.cache),
        ]
    }
}
