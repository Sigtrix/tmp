enum Instr {
    case Char(Character)
    case Match
    case Jump(UInt32)
    case Split(UInt32, UInt32)
}

func recursive_backtracking_vm(_ prog: [Instr], _ pc: UInt32, _ sp: Substring) -> Int32 {
    switch prog[Int(pc)] {
    case let .Char(c):
        if let first = sp.first, c == first {
            let nextIndex = sp.index(after: sp.startIndex)
            return recursive_backtracking_vm(prog, pc + 1, sp[nextIndex...])
        }
        return 0
    case .Match:
        return 1
    case let .Jump(x):
        return recursive_backtracking_vm(prog, x, sp)
    case let .Split(x, y):
        if recursive_backtracking_vm(prog, x, sp) != 0 {
            return 1;
        }
        return recursive_backtracking_vm(prog, y, sp)
    }
}

// TODO: Implement a parser.
// This is the expected bytecode for the regex a+b+.
let prog = [
    Instr.Char("a"),
    Instr.Split(0, 2),
    Instr.Char("b"),
    Instr.Split(2, 4),
    Instr.Match
]

if recursive_backtracking_vm(prog, 0, "aab") != 0 {
    print("Match!")
} else {
    print("Did not match!")
}
