use crate::expr::expr::{Expr, Operator, Tag};
use rand::{rngs::ThreadRng, thread_rng, Rng};

#[derive(Default)]
pub struct Tagger;

pub struct TagState {
    rng: ThreadRng,
}

impl TagState {
    pub fn annotate(&mut self, e: &Expr<()>) -> Expr<Tag> {
        let (a, b, op) = match e {
            Expr::Add(a, b) => (a, b, Expr::Add as Operator),
            Expr::Xor(a, b) => (a, b, Expr::Xor as Operator),
            Expr::RotLeft(a, b) => (a, b, Expr::RotLeft as Operator),
            Expr::RotRight(a, b) => (a, b, Expr::RotRight as Operator),
            Expr::Tag(()) => return Expr::Tag(self.rand_tag()),
        };
        op(Box::new(self.annotate(a)), Box::new(self.annotate(b)))
    }

    fn rand_tag(&mut self) -> Tag {
        match self.rng.gen::<u8>() % 4 {
            0 => Tag::Byte,
            1 => Tag::HashState,
            2..=u8::MAX => Tag::Const(self.rng.gen()),
        }
    }
}

impl Tagger {
    pub fn annotate(&self, e: &Expr<()>) -> Expr<Tag> {
        self.new_tag_state().annotate(e)
    }

    pub fn feedback(&mut self, raw: &Expr<()>, tagged: &Expr<Tag>, score: usize) {}

    pub fn new_tag_state(&self) -> TagState {
        TagState { rng: thread_rng() }
    }
}
