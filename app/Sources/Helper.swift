import Foundation
import Darwin

final class HelperDecider: Decider {
    private let python: String
    private let script: URL
    private let models: URL
    private let lock = NSLock()
    private var process: Process?
    private var input: FileHandle?
    private var outputPipe: Pipe?
    private var outputFD: Int32 = -1
    private var buffer = Data()
    private var starts: [Date] = []
    private var ready = false
    private var starting = false
    private var stale = 0
    var onReady: (() -> Void)?

    init(python: String, script: URL, modelsDir: URL) { self.python = python; self.script = script; self.models = modelsDir; scheduleStart() }

    private func scheduleStart() {
        lock.lock()
        guard !starting else { lock.unlock(); return }
        starting = true
        lock.unlock()
        DispatchQueue.global(qos: .utility).async { [weak self] in self?.start() }
    }

    @discardableResult private func start() -> Bool {
        lock.lock()
        if ready, process?.isRunning == true { starting = false; lock.unlock(); return true }
        starts = starts.filter { Date().timeIntervalSince($0) < 60 }
        guard starts.count < 3 else { starting = false; lock.unlock(); return false }
        starts.append(Date())
        if process?.isRunning == true { process?.terminate() }
        buffer.removeAll(keepingCapacity: true)
        stale = 0
        let p = Process(), to = Pipe(), from = Pipe()
        p.executableURL = URL(fileURLWithPath: python)
        p.arguments = ["-I", script.path, "--models-dir", models.path]
        p.standardInput = to; p.standardOutput = from; p.standardError = FileHandle.nullDevice
        do { try p.run() } catch { starting = false; ready = false; lock.unlock(); return false }
        process = p; input = to.fileHandleForWriting; outputPipe = from
        outputFD = from.fileHandleForReading.fileDescriptor
        _ = fcntl(outputFD, F_SETFL, O_NONBLOCK)
        ready = false
        lock.unlock()

        let deadline = Date().addingTimeInterval(15)
        let handshake = readLine(until: deadline) == "ready"
        var tested = false
        if handshake, let input {
            do { try input.write(contentsOf: Data("P\n".utf8)); tested = readLine(until: deadline) == "pong" }
            catch { tested = false }
        }
        lock.lock()
        let success = handshake && tested && p.isRunning
        ready = success
        starting = false
        if !success { p.terminate() }
        lock.unlock()
        if success { DispatchQueue.main.async { self.onReady?() } }
        return success
    }

    private func takeBufferedLine() -> String? {
        guard let r = buffer.range(of: Data([10])) else { return nil }
        let line = String(data: buffer[..<r.lowerBound], encoding: .utf8) ?? ""
        buffer.removeSubrange(...r.lowerBound)
        return line.trimmingCharacters(in: CharacterSet(charactersIn: "\r"))
    }

    private func readAvailable() {
        guard outputFD >= 0 else { return }
        var fd = pollfd(fd: outputFD, events: Int16(POLLIN), revents: 0)
        guard Darwin.poll(&fd, 1, 0) > 0, fd.revents & Int16(POLLIN) != 0 else { return }
        var bytes = [UInt8](repeating: 0, count: 4096)
        let n = Darwin.read(outputFD, &bytes, bytes.count)
        if n > 0 { buffer.append(contentsOf: bytes[..<n]) }
    }

    private func readLine(until deadline: Date) -> String? {
        while Date() < deadline {
            if let line = takeBufferedLine() { return line }
            var fd = pollfd(fd: outputFD, events: Int16(POLLIN), revents: 0)
            let ms = max(1, Int32(min(30, deadline.timeIntervalSinceNow * 1000)))
            if Darwin.poll(&fd, 1, ms) > 0 {
                var bytes = [UInt8](repeating: 0, count: 4096)
                let n = Darwin.read(outputFD, &bytes, bytes.count)
                if n > 0 { buffer.append(contentsOf: bytes[..<n]) } else if n == 0 { return nil }
            }
        }
        return nil
    }

    private func drainStaleAvailable() {
        while stale > 0 {
            if takeBufferedLine() != nil { stale -= 1; continue }
            let before = buffer.count
            readAvailable()
            if buffer.count == before { break }
        }
    }

    private func readAnswer(until deadline: Date) -> String? {
        while Date() < deadline {
            if let line = takeBufferedLine() {
                if stale > 0 { stale -= 1; continue }
                return line
            }
            var fd = pollfd(fd: outputFD, events: Int16(POLLIN), revents: 0)
            let ms = max(1, Int32(min(30, deadline.timeIntervalSinceNow * 1000)))
            if Darwin.poll(&fd, 1, ms) > 0 {
                var bytes = [UInt8](repeating: 0, count: 4096)
                let n = Darwin.read(outputFD, &bytes, bytes.count)
                if n > 0 { buffer.append(contentsOf: bytes[..<n]) } else if n == 0 { return nil }
            }
        }
        return nil
    }

    func decide(run: String, prev: Lang?, complete: Bool) -> Decision {
        let keep = Decision(convert: false, score: 0, candidate: "", target: .en)
        guard lock.try() else { return keep }
        guard !run.contains("\t"), !run.contains("\n") else { lock.unlock(); return keep }
        guard ready, let p = process, p.isRunning, let input else {
            ready = false
            lock.unlock()
            scheduleStart()
            return keep
        }
        drainStaleAvailable()
        let request = "D\t\(run)\t\(prev?.rawValue ?? "-")\t\(complete ? "1" : "0")\n"
        do { try input.write(contentsOf: Data(request.utf8)) }
        catch { lock.unlock(); return keep }
        guard let line = readAnswer(until: Date().addingTimeInterval(0.03)) else {
            stale += 1
            lock.unlock()
            return keep
        }
        let fields = line.components(separatedBy: "\t")
        let decision: Decision
        if fields.first == "keep", fields.count >= 2 { decision = Decision(convert: false, score: Double(fields[1]) ?? 0, candidate: "", target: .en) }
        else if fields.first == "convert", fields.count >= 4, let score = Double(fields[1]), let target = Lang(rawValue: fields[3]) { decision = Decision(convert: true, score: score, candidate: fields[2], target: target) }
        else { decision = keep }
        lock.unlock()
        return decision
    }

    func stop() {
        lock.lock(); defer { lock.unlock() }
        try? input?.write(contentsOf: Data("Q\n".utf8)); process?.terminate(); ready = false
    }
}
