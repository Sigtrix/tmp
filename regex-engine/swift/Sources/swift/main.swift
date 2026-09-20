enum Instr {
    case Char(Character)
    case Match
    case Jump(UInt32)
    case Split(UInt32, UInt32)
}

func recursiveBacktrackingVM(_ prog: [Instr], _ pc: UInt32, _ sp: Substring) -> Int32 {
    switch prog[Int(pc)] {
    case let .Char(c):
        if let first = sp.first, c == first {
            let nextIndex = sp.index(after: sp.startIndex)
            return recursiveBacktrackingVM(prog, pc + 1, sp[nextIndex...])
        }
        return 0
    case .Match:
        return 1
    case let .Jump(x):
        return recursiveBacktrackingVM(prog, x, sp)
    case let .Split(x, y):
        if recursiveBacktrackingVM(prog, x, sp) != 0 {
            return 1;
        }
        return recursiveBacktrackingVM(prog, y, sp)
    }
}

struct Thread {
    var pc: UInt32
    var sp: Substring
}

func bactrackingVM(_ prog: [Instr], _ data: Substring) -> Int32 {
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

struct ThompsonThread {
    var pc: UInt32
}

func addThread(
    _ list: inout [ThompsonThread],
    _ seen: inout Set<UInt32>,
    _ pc: UInt32
) {
    if !seen.insert(pc).inserted {
        return
    }

    list.append(ThompsonThread(pc: pc))
}

func thompsonVM(_ prog: [Instr], _ input: Substring) -> Int32 {
    var currSeen = Set<UInt32>()
    var nextSeen = Set<UInt32>()
    
    var currThreadList: [ThompsonThread] = []
    var nextThreadList: [ThompsonThread] = []
    addThread(&currThreadList, &currSeen, 0)

    var iter = input.makeIterator()
    while true {
        let sp = iter.next()

        var i = 0
        while i < currThreadList.count {
            let pc = currThreadList[i].pc
            i += 1

            switch prog[Int(pc)] {
            case let .Char(c):
                if sp != c {
                    break
                }
                addThread(&nextThreadList, &nextSeen, pc+1)
                break
            case .Match:
                return 1
            case let .Jump(x):
                addThread(&currThreadList, &currSeen, x)
                break
            case let .Split(x, y):
                addThread(&currThreadList, &currSeen, x)
                addThread(&currThreadList, &currSeen, y)
                break
            }
        }

        if sp == nil {
            return 0
        }
        swap(&currThreadList, &nextThreadList)
        swap(&currSeen, &nextSeen)
        nextThreadList.removeAll(keepingCapacity: true)
        nextSeen.removeAll(keepingCapacity: true)
    }
}

// TODO: Move to tests.

// TODO: Implement a parser.
// This is the expected bytecode for the regex a+b+.
let prog = [
    Instr.Char("a"),
    Instr.Split(0, 2),
    Instr.Char("b"),
    Instr.Split(2, 4),
    Instr.Match
]

if recursiveBacktrackingVM(prog, 0, "aab") != 0 {
    print("Matched by the recursive backtracking VM!")
} else {
    print("Did not match by the recursive backtracking VM!")
}


if bactrackingVM(prog, "aab") != 0 {
    print("Matched by backtracking VM!")
} else {
    print("Did not match by the backtracking VM!")
}


if thompsonVM(prog, "aab") != 0 {
    print("Matched by the thompson VM!")
} else {
    print("Did not match by the thompson VM!")
}


// Compare the time taken to match a^n by a?^na^n
// by a backtracking VM (requring O(2^n) ops) and
// the thompson (no backtrackin) VM (requiring O(mn)) ops.

// Construct the string a^n.
func makeTimingInput(n: Int) -> String {
    String(repeating: "a", count: n)
}

// Construct the bytecode program for the regex a?^na^n.
func makeTimingProg(n: Int) -> [Instr] {
    var prog: [Instr] = []

    for _ in 0..<n {
        let split = UInt32(prog.count)
        prog.append(.Split(split + 1, split + 2))
        prog.append(.Char("a"))
    }

    for _ in 0..<n {
        prog.append(.Char("a"))
    }

    prog.append(.Match)

    return prog
}

let n = 20
let timingProg = makeTimingProg(n: n)
let timingInput = makeTimingInput(n: n)

let backtrackingClock = ContinuousClock()
let backtrackingStart = backtrackingClock.now
let backtrackingResult = bactrackingVM(timingProg, timingInput[...]) 
let backtrackingElapsed = backtrackingClock.now - backtrackingStart
if backtrackingResult != 0 {
    print("Matched by backtracking VM in \(backtrackingElapsed)")
} else {
    print("Did not match by the backtracking VM!")
}


let thompsonClock = ContinuousClock()
let thompsonStart = thompsonClock.now
let thompsonResult = thompsonVM(timingProg, timingInput[...]) 
let thompsonElapsed = thompsonClock.now - thompsonStart
if thompsonResult != 0 {
    print("Matched by the thompson VM in \(thompsonElapsed)")
} else {
    print("Did not match by the thompson VM!")
}

// Time the programs on a range of values for n and write the results
// to a file in CSV format.
import Foundation

struct TimingResult {
    let n: Int
    let backtracking: Duration
    let thompson: Duration
}

func timeBacktracking(n: Int) -> Duration {
    let timingProg = makeTimingProg(n: n)
    let timingInput = makeTimingInput(n: n)

    let clock = ContinuousClock()
    let start = clock.now

    let result = bactrackingVM(timingProg, timingInput[...])

    let elapsed = clock.now - start

    if result == 0 {
        print("Did not match by the backtracking VM for n = \(n)")
    }

    return elapsed
}

func timeThompson(n: Int) -> Duration {
    let timingProg = makeTimingProg(n: n)
    let timingInput = makeTimingInput(n: n)

    let clock = ContinuousClock()
    let start = clock.now

    let result = thompsonVM(timingProg, timingInput[...])

    let elapsed = clock.now - start

    if result == 0 {
        print("Did not match by the Thompson VM for n = \(n)")
    }

    return elapsed
}

func durationToNanoseconds(_ duration: Duration) -> Int64 {
    let components = duration.components

    // Duration stores seconds + attoseconds.
    let seconds = Int64(components.seconds)
    let attoseconds = components.attoseconds

    return seconds * 1_000_000_000
        + attoseconds / 1_000_000_000
}

// Overwrites the file if it already exists.
func writeTimingsCSV(_ results: [TimingResult], to path: String) throws {
    var csv = "n,backtracking_ns,thompson_ns\n"

    for result in results {
        csv += "\(result.n),"
        csv += "\(durationToNanoseconds(result.backtracking)),"
        csv += "\(durationToNanoseconds(result.thompson))\n"
    }

    try csv.write(
        toFile: path,
        atomically: true,
        encoding: .utf8
    )
}


// MARK: - Run benchmark

var timings: [TimingResult] = []

for n in 1..<25 {
    print("Running n = \(n)...")

    let backtracking = timeBacktracking(n: n)
    let thompson = timeThompson(n: n)

    let result = TimingResult(
        n: n,
        backtracking: backtracking,
        thompson: thompson
    )

    timings.append(result)

    print("  backtracking: \(backtracking)")
    print("  thompson:     \(thompson)")
}

let csvPath = "timings.csv"

do {
    try writeTimingsCSV(timings, to: csvPath)
    print("Wrote \(csvPath)")
} catch {
    print("Failed to write CSV: \(error)")
}
