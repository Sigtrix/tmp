enum Instr {
    Char(char),
    Match,
    Jump(usize),
    Split(usize, usize),
}

fn recursive_backtracking_vm(prog: &[Instr], pc: usize, sp: &str) -> i32 {
    match &prog[pc] {
        Instr::Char(c) => {
            if let Some(ch) = sp.chars().next() && *c == ch {
                return recursive_backtracking_vm(
                    prog,
                    pc + 1,
                    &sp[ch.len_utf8()..],
                );
            }
            0
        },
        Instr::Match => 1,
        Instr::Jump(x) => recursive_backtracking_vm(prog, *x, sp),
        Instr::Split(x, y) => { 
            if recursive_backtracking_vm(prog, *x, sp) != 0 {
                return 1;
            }
            recursive_backtracking_vm(prog, *y, sp)
        },
    }
}

fn main() {
    // TODO: Implement a parser. 
    // This is the expected bytecode for the regex a+b+.
    let prog: Vec<Instr> = vec![
        Instr::Char('a'), // TODO(config): Change config chars should not be red.
        Instr::Split(0, 2),
        Instr::Char('b'),
        Instr::Split(2, 4),
        Instr::Match,
    ];

    // TODO: Accept the string (as well as the regex to parse etc.) from the user.
    if recursive_backtracking_vm(&prog, 0, "aab") != 0 {
        println!{"Match!"}
    } else {
        println!{"Did not match!"}
    }
}
