use std::io::stdin;
use std::ascii::Ascii;

pub struct Translator {
    reader: Box<Reader>,
    look: Ascii,
    labels: uint,
}

impl Translator {
    pub fn init() -> Translator {
        let mut t = Translator {
            look: '\0'.to_ascii(),
            reader: box stdin(),
            labels: 0,
        };
        t.read(); // this is important! reads the first char of input
        t
    }

    /// Get the current lookahead character
    pub fn look(&self) -> char {
        self.look.to_char()
    }

    /// Read the next character of input
    pub fn read(&mut self) {
        self.look = self.reader.read_byte().ok()
                        .expect("expected another character").to_ascii();
    }

    /// Check if the current character is `c`, fail otherwise
    pub fn match_(&mut self, c: char) {
        if self.look == c.to_ascii() {
            self.read();
        } else {
            expected(c.to_str().as_slice());
        }
    }

    /// Get an identifier
    pub fn get_name(&mut self) -> Ascii {
        let l = self.look;
        if !l.is_alphabetic() {
            expected("Name");
        }
        self.read();
        l.to_uppercase()
    }

    /// Generate a unique label
    fn new_label(&mut self) -> String {
        let res = format!("L{}", self.labels);
        self.labels += 1;
        res
    }

    /// Post a label to output
    fn post_label(&self, label: &str) {
        emit(format!("{}:", label).as_slice());
    }

    /// <program> ::= <block> END
    pub fn program(&mut self) {
        self.block();
        if self.look.to_char() != 'e' {
            expected("End");
        }
        emitln("END");
    }

    /// <block> ::= [ <statement> ]*
    /// <statement> ::= <if> | <while> | <loop> | <repeat> | <for> | <do> | <other>
    fn block(&mut self) {
        loop {
            match self.look.to_char() {
                'i' => self.if_(),
                'w' => self.while_(),
                'p' => self.loop_(),
                'r' => self.repeat(),
                'f' => self.for_(),
                'd' => self.do_(),
                'e' | 'l' | 'u' => return,
                _   => self.other()
            }
        }
    }

    /// <if> ::= i <condition> <block> l <block> e
    fn if_(&mut self) {
        self.match_('i');

        let label1 = self.new_label();
        let mut label2 = label1.clone();

        self.condition();

        emitln(format!("JZ {}", label1).as_slice());

        self.block();

        if self.look.to_char() == 'l' {
            self.match_('l');

            label2 = self.new_label();
            emitln(format!("JMP {}", label2).as_slice());

            self.post_label(label1.as_slice());

            self.block()
        }

        self.match_('e');

        self.post_label(label2.as_slice());
    }

    /// <while> ::= w <condition> <block> e
    fn while_(&mut self) {
        self.match_('w');
        let label1 = self.new_label();
        let label2 = self.new_label();

        self.post_label(label1.as_slice());

        self.condition();

        emitln(format!("JZ {}", label2).as_slice());

        self.block();

        self.match_('e');

        emitln(format!("JMP {}", label1).as_slice());

        self.post_label(label2.as_slice());
    }

    /// <loop> ::= p <block> e
    fn loop_(&mut self) {
        self.match_('p');

        let label = self.new_label();
        self.post_label(label.as_slice());

        self.block();

        self.match_('e');
        emitln(format!("JMP {}", label).as_slice());
    }

    /// <repeat> ::= r <block> u <condition>
    fn repeat(&mut self) {
        self.match_('r');

        let label = self.new_label();
        self.post_label(label.as_slice());

        self.block();

        self.match_('u');

        self.condition();

        emitln(format!("JZ {}", label).as_slice());
    }

    /// <for> ::= f <name> = <expr> <expr> <block> e
    fn for_(&mut self) {
        emitln("PUSH EBX");

        self.match_('f');
        let label1 = self.new_label();
        let label2 = self.new_label();

        let name = self.get_name();
        self.match_('=');

        emitln(format!("<somehow load {}>", name).as_slice());

        self.expression();
        emitln("MOV EBX, EAX");

        self.expression();

        emitln("SUB EAX, EBX");
        emitln(format!("JO {}", label2).as_slice());
        emitln(format!("<somehow store EAX to {}>", name).as_slice());

        self.post_label(label1.as_slice());

        self.block();

        self.match_('e');

        emitln(format!("<somehow SUB {}, 1>", name).as_slice());
        emitln(format!("JNZ {}", label1).as_slice());

        self.post_label(label2.as_slice());
        emitln("POP EBX");
    }

    /// <do> = d <expr> <block> e
    fn do_(&mut self) {
        self.match_('d');
        let label = self.new_label();

        self.expression();
        emitln("MOV ECX, EAX");

        self.post_label(label.as_slice());
        self.block();

        emitln(format!("LOOP {}", label).as_slice());

        self.match_('e');
    }

    /// <other> ::= <name>
    fn other(&mut self) {
        emitln(self.get_name().to_str().as_slice());
    }

    fn condition(&mut self) {
        emitln("<condition>");
    }

    fn expression(&mut self) {
        emitln("<expression>");
    }
}

/// Report error
pub fn error(s: &str) {
    println!("Error: {}.", s);
}

/// Report error and exit
pub fn abort(s: &str) -> ! {
    error(s);
    fail!();
}

/// Report what was expected and exit
pub fn expected(s: &str) -> ! {
    println!("Error: {} expected.", s);
    fail!();
}

/// Output a string with tab
pub fn emit(s: &str) {
    print!("\t{}", s);
}

/// Output a string with tab and newlnie
pub fn emitln(s: &str) {
    println!("\t{}", s);
}
