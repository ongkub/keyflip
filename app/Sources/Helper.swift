import Foundation
import Darwin

final class HelperDecider: Decider {
    private let python: String; private let script: URL; private let models: URL
    private let lock = NSLock(); private var process: Process?; private var input: FileHandle?; private var outputPipe: Pipe?; private var outputFD: Int32 = -1; private var buffer = Data(); private var starts: [Date] = []; private var ready = false; private var starting = false
    var onReady: (() -> Void)?
    init(python: String, script: URL, modelsDir: URL) { self.python = python; self.script = script; self.models = modelsDir; scheduleStart() }
    private func scheduleStart() { guard !starting else { return }; starting = true; DispatchQueue.global(qos: .utility).async { [weak self] in guard let self else { return }; self.lock.lock(); _ = self.start(); self.starting = false; self.lock.unlock() } }
    private func start() -> Bool {
        if ready, process?.isRunning == true { return true }
        starts = starts.filter { Date().timeIntervalSince($0) < 60 }; guard starts.count < 3 else { return false }; starts.append(Date())
        if process?.isRunning == true { process?.terminate() }
        let p = Process(), to = Pipe(), from = Pipe(); p.executableURL = URL(fileURLWithPath: python); p.arguments = ["-I", script.path, "--models-dir", models.path]; p.standardInput = to; p.standardOutput = from; p.standardError = FileHandle.nullDevice
        do { try p.run() } catch { return false }
        process = p; input = to.fileHandleForWriting; outputPipe = from; outputFD = from.fileHandleForReading.fileDescriptor; _ = fcntl(outputFD, F_SETFL, O_NONBLOCK)
        let deadline = Date().addingTimeInterval(15)
        while Date() < deadline { if let line = readLine(until: deadline), line == "ready" { ready = true; DispatchQueue.main.async { self.onReady?() }; return true }; if !p.isRunning { break } }
        p.terminate(); ready = false; return false
    }
    private func readLine(until deadline: Date) -> String? {
        while Date() < deadline {
            if let r = buffer.range(of: Data([10])) { let line = String(data: buffer[..<r.lowerBound], encoding: .utf8) ?? ""; buffer.removeSubrange(...r.lowerBound); return line.trimmingCharacters(in: CharacterSet(charactersIn: "\r")) }
            var fd = pollfd(fd: outputFD, events: Int16(POLLIN), revents: 0); let ms = max(1, Int32(min(30, deadline.timeIntervalSinceNow * 1000))); if Darwin.poll(&fd, 1, ms) > 0 { var bytes = [UInt8](repeating: 0, count: 4096); let n = Darwin.read(outputFD, &bytes, bytes.count); if n > 0 { buffer.append(contentsOf: bytes[..<n]) } else if n == 0 { return nil } }
        }
        return nil
    }
    func decide(run: String, prev: Lang?, complete: Bool) -> Decision {
        lock.lock(); defer { lock.unlock() }
        let keep = Decision(convert: false, score: 0, candidate: "", target: .en)
        guard !run.contains("\t"), !run.contains("\n") else { return keep }
        guard ready, process?.isRunning == true, let input = input else { ready = false; scheduleStart(); return keep }
        let request = "D\t\(run)\t\(prev?.rawValue ?? "-")\t\(complete ? "1" : "0")\n"
        do { try input.write(contentsOf: Data(request.utf8)) } catch { ready = false; return keep }
        guard let line = readLine(until: Date().addingTimeInterval(0.03)) else { ready = false; scheduleStart(); return keep }
        let fields = line.components(separatedBy: "\t")
        if fields.first == "keep", fields.count >= 2 { return Decision(convert: false, score: Double(fields[1]) ?? 0, candidate: "", target: .en) }
        if fields.first == "convert", fields.count >= 4, let score = Double(fields[1]), let target = Lang(rawValue: fields[3]) { return Decision(convert: true, score: score, candidate: fields[2], target: target) }
        return keep
    }
    func stop() { lock.lock(); defer { lock.unlock() }; try? input?.write(contentsOf: Data("Q\n".utf8)); process?.terminate(); ready = false }
}
