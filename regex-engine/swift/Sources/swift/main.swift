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

struct Thread {
    var pc: UInt32
    var sp: Substring
}

func backtracking_vm(_ prog: [Instr], _ data: Substring) -> Int32 {
    let MAX_THREADS = 1000
    var ready: [Thread] = [] 
    ready.append(Thread(pc: 0, sp: data))

    while(!ready.isEmpty) {
        let thread = ready.removeLast()
        var pc = thread.pc
        var sp = thread.sp

        var done = false
        while !done {
            switch prog[Int(pc)] {        
            case let .Char(c):
                if let first = sp.first, c == first {
                    pc += 1
                    let nextIndex = sp.index(after: sp.startIndex)
                    sp = sp[nextIndex...]
                    continue
                }
                done = true
            case .Match:
                return 1
            case let .Jump(x):
                pc = x
                continue
            case let .Split(x, y):
                if ready.count >= MAX_THREADS {
                    print("Regex overflow")
                    return -1
                }
                ready.append(Thread(pc: y, sp: sp))
                pc = x
                continue
            }
        }
    }
    return 0
}


if recursive_backtracking_vm(prog, 0, "aab") != 0 {
    print("Matched by the recursive backtracking VM!")
} else {
    print("Did not match by the recursive backtracking VM!")
}


if backtracking_vm(prog, "aab") != 0 {
    print("Matched by backtracking VM!")
} else {
    print("Did not match by the backtracking VM!")
}
