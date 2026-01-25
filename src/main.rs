use std::fmt;
use std::str::FromStr;

type Atom = usize;

#[derive(Debug, Copy, Clone)]
struct Literal(isize);

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Literal {
    fn atom(&self) -> Atom {
        self.0.unsigned_abs()
    }

    fn is_positive(&self) -> bool {
        self.0 > 0
    }
}

#[derive(Debug, Clone)]
struct Clause(Vec<Literal>);

impl FromStr for Clause {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, String> {
        let mut lits: Vec<Literal> = Vec::new();
        for word in input.split(' ') {
            match word.parse::<isize>() {
                Err(e) => return Err(format!("Could not parse literal: {}", e.to_string())),
                Ok(0) => break,
                Ok(value) => lits.push(Literal(value)),
            };
        }
        Ok(Clause(lits))
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for lit in &self.0 {
            write!(f, "{} ", lit)?;
        }
        write!(f, "0")
    }
}

impl Clause {
    fn resolve_mut(&mut self, _other: &Clause) {
        todo!()
    }
}

#[derive(Debug, Clone)]
struct Formula(Vec<Clause>);

impl Formula {
    // Calculate number of variables (max absolute value of any literal)
    fn num_vars(&self) -> usize {
        self.0
            .iter()
            .flat_map(|clause| clause.0.iter())
            .map(|lit| lit.atom())
            .max()
            .unwrap_or(0)
    }
}

impl FromStr for Formula {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, String> {
        let mut clauses: Vec<Clause> = Vec::new();
        for line in input.lines() {
            match line.chars().next() {
                Some('p') => {} // Formula size. Can ignore
                Some('c') => {} // comment
                None => {}
                _ => clauses.push(line.parse::<Clause>()?),
            }
        }
        Ok(Self(clauses))
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num_clauses = self.0.len();

        writeln!(f, "p cnf {} {}", self.num_vars(), num_clauses)?;
        for clause in &self.0 {
            writeln!(f, "{}", clause)?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum Reason {
    Decision { level: u32 },
    Implied { level: u32, antecedent: u32 },
}

impl Reason {
    fn level(&self) -> u32 {
        match *self {
            Reason::Decision { level } | Reason::Implied { level, .. } => level,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct VarData {
    value: bool,
    reason: Reason,
}

#[derive(Debug, Clone)]
struct Valuation(Vec<Option<VarData>>);

#[derive(Debug, Clone)]
struct Lilsat {
    formula: Formula,
    valuation: Valuation,
}

#[derive(Debug, Clone)]
enum Answer {
    SAT(Valuation),
    UNSAT,
}

type ClauseIdx = usize;

impl Lilsat {
    fn learn_literal(&mut self, lit: Literal, reason: Reason) {
        self.valuation.0[lit.atom()] = Some(VarData {
            value: lit.is_positive(),
            reason,
        })
    }

    fn learn_clause(&mut self, clause: Clause) {
        self.formula.0.push(clause)
    }

    fn eval_lit(&self, lit: Literal) -> Option<bool> {
        self.valuation.0[lit.atom()].map(|d: VarData| d.value)
    }

    fn choose_lit(&self) -> Option<Literal> {
        for clause in &self.formula.0 {
            for lit in &clause.0 {
                if let None = self.eval_lit(*lit) {
                    return Some(*lit);
                }
            }
        }
        return None;
    }

    fn unit_propagate(&self) -> Result<(), usize> {
        todo!()
    }

    fn get_antecedent_1uip(&self, _level: u32, _conflict_clause: &Clause) -> Option<ClauseIdx> {
        todo!()
    }

    fn analyze_conflict(&self, level: u32, clause: &mut Clause) -> u32 {
        while let Some(antecedent_idx) = self.get_antecedent_1uip(level, clause) {
            clause.resolve_mut(&self.formula.0[antecedent_idx])
        }
        todo!("Find backtracking level")
    }

    fn backtrack_to(&mut self, level: u32) {
        for maybe_var_data in self.valuation.0.iter_mut() {
            if let Some(var_data) = maybe_var_data {
                if var_data.reason.level() >= level {
                    *maybe_var_data = None;
                }
            }
        }
    }

    fn run(&mut self) -> Answer {
        let mut level: u32 = 0;
        while let Some(lit) = self.choose_lit() {
            self.learn_literal(lit, Reason::Decision { level });
            if let Err(conflict_idx) = self.unit_propagate() {
                let mut learn_clause = self.formula.0[conflict_idx].clone();
                level = self.analyze_conflict(level, &mut learn_clause);
                self.learn_clause(learn_clause);
                self.backtrack_to(level);
                if let Err(_) = self.unit_propagate() {
                    return Answer::UNSAT;
                }
            } else {
                level += 1;
            }
        }
        Answer::SAT(self.valuation.clone())
    }

    fn solve(formula: &Formula) -> Answer {
        let mut lilsat = Lilsat {
            formula: formula.clone(),
            valuation: Valuation(vec![None; formula.num_vars()]),
        };
        lilsat.run()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <cnf-file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    let contents = std::fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading file '{}': {}", filename, err);
        std::process::exit(1);
    });

    let formula: Formula = contents.parse().unwrap_or_else(|err| {
        eprintln!("Error parsing CNF: {}", err);
        std::process::exit(1);
    });

    println!("{:?}", Lilsat::solve(&formula));
}
