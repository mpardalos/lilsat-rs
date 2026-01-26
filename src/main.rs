use std::fmt;
use std::str::FromStr;

type Atom = usize;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    fn negation(&self) -> Self {
        Literal(-self.0)
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
        let mut first = true;
        for lit in &self.0 {
            if !first {
                write!(f, " ∨ ")?;
            }
            write!(f, "{}", lit)?;
            first = false;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum ClauseDecision {
    SAT,
    UNSAT,
    Undecided,
    Unit(Literal),
}

impl Clause {
    fn resolve_mut(&mut self, other: &Clause) {
        // println!("({}) ⊙= ({})", self, other);
        for lit in other.0.iter() {
            if self.lit_position(*lit).is_some() { /* Do nothing */
            } else if let Some(pos) = self.lit_position(lit.negation()) {
                self.0.swap_remove(pos);
            } else {
                self.0.push(*lit)
            }
        }
    }

    fn lit_position(&self, lit: Literal) -> Option<usize> {
        self.0.iter().position(|l| lit == *l)
    }

    fn decide(&self, valuation: &Valuation) -> ClauseDecision {
        let mut decision = ClauseDecision::UNSAT;
        for lit in self.0.iter() {
            decision = match (decision, valuation.eval_lit(*lit)) {
                (_, Some(true)) => ClauseDecision::SAT,
                (_, Some(false)) => decision,
                (ClauseDecision::SAT, _) => ClauseDecision::SAT,
                (ClauseDecision::UNSAT, None) => ClauseDecision::Unit(*lit),
                (ClauseDecision::Undecided, _) => ClauseDecision::Undecided,
                (ClauseDecision::Unit(_), None) => ClauseDecision::Undecided,
            };
        }
        decision
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
            .map(|x| x + 1)
            .unwrap_or(0)
    }
}

impl FromStr for Formula {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, String> {
        let mut clauses: Vec<Clause> = Vec::new();
        for line in input.lines() {
            match line.trim().chars().next() {
                Some('p') => {} // Formula size. Can ignore
                Some('c') => {} // comment
                Some('%') => {} // idk, but ignore
                Some('0') => {} // Empty clause
                None => {}
                _ => clauses.push(line.trim().parse::<Clause>()?),
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
    Implied { level: u32, antecedent: ClauseIdx },
}

impl Reason {
    fn level(&self) -> u32 {
        match *self {
            Reason::Decision { level } | Reason::Implied { level, .. } => level,
        }
    }

    fn antecedent(&self) -> Option<ClauseIdx> {
        match *self {
            Reason::Implied { antecedent, .. } => Some(antecedent),
            _ => None,
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

impl fmt::Display for Valuation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (var, maybe_var_data) in self.0.iter().enumerate() {
            if let Some(var_data) = maybe_var_data {
                writeln!(
                    f,
                    "{}: {}@{}",
                    var,
                    if var_data.value { "⊤" } else { "⊥" },
                    var_data.reason.level()
                )?;
            }
        }
        Ok(())
    }
}

impl Valuation {
    fn eval_lit(&self, lit: Literal) -> Option<bool> {
        let atom_value = self.0[lit.atom()].map(|d: VarData| d.value);
        if lit.0 < 0 {
            atom_value.map(|v| !v)
        } else {
            atom_value
        }
    }

    fn var_data(&self, var: Atom) -> Option<VarData> {
        self.0[var]
    }

    fn learn_literal(&mut self, lit: Literal, reason: Reason) {
        self.0[lit.atom()] = Some(VarData {
            value: lit.is_positive(),
            reason,
        })
    }
}

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

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UNSAT => write!(f, "UNSAT")?,
            Self::SAT(valuation) => {
                writeln!(f, "SAT")?;
                write!(f, "{}", valuation)?;
            }
        }
        Ok(())
    }
}

type ClauseIdx = usize;

impl fmt::Display for Lilsat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (var, maybe_var_data) in self.valuation.0.iter().enumerate() {
            if let Some(var_data) = maybe_var_data {
                write!(
                    f,
                    "{}: {}@{}",
                    var,
                    if var_data.value { "⊤" } else { "⊥" },
                    var_data.reason.level()
                )?;
                if let Some(idx) = var_data.reason.antecedent() {
                    write!(f, " by ({})", &self.formula.0[idx])?;
                } else {
                    write!(f, " by decisision")?;
                }
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

impl Lilsat {
    fn learn_clause(&mut self, clause: Clause) {
        self.formula.0.push(clause)
    }

    fn choose_lit(&self) -> Option<Literal> {
        for clause in &self.formula.0 {
            for lit in &clause.0 {
                if let None = self.valuation.eval_lit(*lit) {
                    return Some(*lit);
                }
            }
        }
        return None;
    }

    fn unit_propagate(&mut self, level: u32) -> Result<(), ClauseIdx> {
        loop {
            let mut changed = false;
            for (idx, clause) in self.formula.0.iter().enumerate() {
                match clause.decide(&self.valuation) {
                    ClauseDecision::SAT => {}
                    ClauseDecision::UNSAT => return Err(idx),
                    ClauseDecision::Undecided => {}
                    ClauseDecision::Unit(literal) => {
                        // println!("{} -> {}@{}", self.formula.0[idx], literal, level);
                        self.valuation.learn_literal(
                            literal,
                            Reason::Implied {
                                level,
                                antecedent: idx,
                            },
                        );
                        // println!("---\n{}", self);
                        changed = true;
                    }
                }
            }
            if !changed {
                return Ok(());
            }
        }
    }

    fn get_antecedent_1uip(&self, level: u32, conflict_clause: &Clause) -> Option<ClauseIdx> {
        let mut chosen: Option<ClauseIdx> = None;
        let mut at_current_level = 0;
        for lit in conflict_clause.0.iter() {
            if let Some(d) = self.valuation.var_data(lit.atom()) {
                if d.reason.level() == level {
                    at_current_level += 1;
                    if let Some(antecedent) = d.reason.antecedent() {
                        chosen = Some(antecedent)
                    }
                }
            }
        }
        match at_current_level {
            0 => panic!("No available pivot"),
            1 => None,
            _ => chosen,
        }
    }

    fn analyze_conflict(&self, level: u32, clause: &mut Clause) -> u32 {
        while let Some(antecedent_idx) = self.get_antecedent_1uip(level, clause) {
            clause.resolve_mut(&self.formula.0[antecedent_idx]);
        }
        clause
            .0
            .iter()
            .map(|lit| {
                self.valuation.0[lit.atom()]
                    .expect("Undecided variable in conflict clause")
                    .reason
                    .level()
            })
            .min()
            .expect("Empty conflict clause")
    }

    fn backtrack_to(&mut self, level: u32) {
        // println!("Backtrack to {}", level);
        for (_var, maybe_var_data) in self.valuation.0.iter_mut().enumerate() {
            if let Some(var_data) = maybe_var_data {
                if var_data.reason.level() >= level {
                    // println!("  Forget {}@{}", var, var_data.reason.level());
                    *maybe_var_data = None;
                }
            }
        }
    }

    fn run(&mut self) -> Answer {
        let mut level: u32 = 0;
        while let Some(lit) = self.choose_lit() {
            self.valuation
                .learn_literal(lit, Reason::Decision { level });
            // println!("Decide {}@{}", lit, level);
            if let Err(conflict_idx) = self.unit_propagate(level) {
                let mut learn_clause = self.formula.0[conflict_idx].clone();
                level = self.analyze_conflict(level, &mut learn_clause);
                self.learn_clause(learn_clause);
                self.backtrack_to(level);
                if let Err(_) = self.unit_propagate(level) {
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

    println!("{}", Lilsat::solve(&formula));
}
