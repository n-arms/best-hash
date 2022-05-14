use crate::expr::expr::Expr;
use std::collections::VecDeque;
use std::collections::HashSet;
use std::fmt;

pub struct Search {
    to_visit: VecDeque<Expr<()>>,
    visited: HashSet<Expr<()>>
}

impl Default for Search {
    fn default() -> Self {
        Search {
            to_visit: VecDeque::from([Expr::Tag(())]),
            visited: HashSet::new()
        }
    }
}

fn closest_leaf<TAG>(expr: &Expr<TAG>) -> usize {
    match expr {
        Expr::Add(a, b) |
        Expr::Xor(a, b) |
        Expr::RotLeft(a, b) |
        Expr::RotRight(a, b) => closest_leaf(&a).min(closest_leaf(&b)) + 1,
        Expr::Tag(_) => 0,
    }
}

/// returns the closest leaf to the root in the form of a u8, as well as the path to get to it, in
/// the form of a u128 where 0 means left and 1 means right
fn leaf_path(expr: &Expr<()>) -> (u8, u128) {
    match expr {
        Expr::Add(a, b) |
        Expr::Xor(a, b) |
        Expr::RotLeft(a, b) |
        Expr::RotRight(a, b) => {
            let (a_len, a_path) = leaf_path(&a);
            let (b_len, b_path) = leaf_path(&b);

            if a_len < b_len {
                (a_len + 1, a_path << 1)
            } else {
                (b_len + 1, (b_path << 1) + 1)
            }
        }
        Expr::Tag(_) => (0, 0),
    }
}

fn leafify(f: fn(Box<Expr<()>>, Box<Expr<()>>) -> Expr<()>) -> Expr<()> {
    f(Box::new(Expr::Tag(())), Box::new(Expr::Tag(())))
}

fn binary_permutations(a: &Expr<()>, b: &Expr<()>, depth: u8, path: u128, f: fn(Box<Expr<()>>, Box<Expr<()>>) -> Expr<()>) -> Vec<Expr<()>> {
    let mut perms = Vec::new();
    if path & 1 == 0 {
        let a_perms = permutations(&a, depth - 1, path >> 1);
        for ap in a_perms {
            perms.push(f(Box::new(ap), Box::new(b.clone())));
        }
    } else {
        let b_perms = permutations(&b, depth - 1, path >> 1);
        for bp in b_perms {
            perms.push(f(Box::new(a.clone()), Box::new(bp)));
        }
    }
    perms
}

fn permutations(expr: &Expr<()>, depth: u8, path: u128) -> Vec<Expr<()>> {
    if depth == 0 {
        if let Expr::Tag(()) = expr {
            vec![
                leafify(Expr::Add),
                leafify(Expr::Xor),
                leafify(Expr::RotLeft),
                leafify(Expr::RotRight)
            ]
        } else {
            vec![expr.clone()]
        }
    } else {
        match expr {
            Expr::Add(a, b) => binary_permutations(a, b, depth, path, Expr::Add),
            Expr::Xor(a, b) => binary_permutations(a, b, depth, path, Expr::Xor),
            Expr::RotLeft(a, b) => binary_permutations(a, b, depth, path, Expr::RotLeft),
            Expr::RotRight(a, b) => binary_permutations(a, b, depth, path, Expr::RotRight),
            Expr::Tag(()) => vec![Expr::Tag(())]
        }
    }
}

impl Iterator for Search {
    type Item = Expr<()>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut visiting = self.to_visit.pop_front()?;
        while self.visited.contains(&visiting) {
            visiting = self.to_visit.pop_front()?;
        }
        self.visited.insert(visiting.clone());

        let (depth, path) = leaf_path(&visiting);
        let new = permutations(&visiting, depth, path);

        for expr in new {
            if !self.visited.contains(&expr) {
                self.to_visit.push_back(expr);
            }
        }

        Some(visiting)
    }
}

impl fmt::Display for Search {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        for expr in &self.to_visit {
            writeln!(f, "\t{}", expr)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl Search {
    pub fn len(&self) -> usize {
        self.to_visit.len()
    }
}
