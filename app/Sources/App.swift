import AppKit
import ApplicationServices
import Carbon

// No nib: create the delegate ourselves (NSApplicationMain would never instantiate it).
@main enum KeyFlipMain {
    static let delegate = AppDelegate()
    static func main() { let app = NSApplication.shared; app.delegate = delegate; app.run() }
}

final class AppDelegate: NSObject, NSApplicationDelegate, NSMenuDelegate {
    private let store = Store()
    private var tracker: EngineBridge!
    private var tap: KeyTap!
    private var status: NSStatusItem!
    private var toggle: NSMenuItem!
    private var info: NSMenuItem!
    private var modelInfo: NSMenuItem!
    private var reportItem: NSMenuItem!
    private var clearItem: NSMenuItem!
    private var trustedTimer: Timer?
    private var reportWindow: ReportWindowController?
    private var storedNever: Set<String> = []
    private var isTrusted = false
    private var engineReady = false
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)
        let resources = Bundle.main.resourceURL ?? URL(fileURLWithPath: ".")
        tracker = EngineBridge(modelsDir: store.models(bundleResources: resources), config: ["deleteMode": store.settings["deleteMode"] as? String ?? "scalar", "never": Array(store.never)])
        engineReady = tracker.error == nil
        if let error = tracker.error { store.debug("Rust engine unavailable: \(error)") }
        storedNever = tracker.never
        tap = KeyTap(tracker: tracker, store: store); tap.enabled = store.settings["enabled"] as? Bool ?? true
        tracker.onEvents = { [weak self] event in self?.record(event) }
        buildMenu(); askTrust()
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(appChanged), name: NSWorkspace.didActivateApplicationNotification, object: nil)
        DistributedNotificationCenter.default().addObserver(self, selector: #selector(inputChanged), name: Notification.Name(kTISNotifySelectedKeyboardInputSourceChanged as String), object: nil)
    }
    private func buildMenu() {
        status = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength); status.button?.title = "ก⇄A ⚠"
        let menu = NSMenu(); menu.delegate = self
        toggle = NSMenuItem(title: "แก้ภาษาอัตโนมัติ", action: #selector(toggleEnabled), keyEquivalent: ""); toggle.target = self; menu.addItem(toggle)
        info = NSMenuItem(title: "วันนี้: พิมพ์ 0 คำ · แก้ให้ 0 · ย้อน 0", action: nil, keyEquivalent: ""); info.isEnabled = false; menu.addItem(info)
        modelInfo = NSMenuItem(title: "โมเดล: ทั่วไป", action: nil, keyEquivalent: ""); modelInfo.isEnabled = false; menu.addItem(modelInfo)
        menu.addItem(.separator()); clearItem = NSMenuItem(title: "ล้างคำที่ตั้งไม่ให้แปลง", action: #selector(clearNever), keyEquivalent: ""); clearItem.target = self; menu.addItem(clearItem); reportItem = NSMenuItem(title: "แจ้งปัญหา…", action: #selector(showReport), keyEquivalent: ""); reportItem.target = self; menu.addItem(reportItem); add(menu,"เปิดโฟลเดอร์ข้อมูล",#selector(openData)); add(menu,"ตั้งค่าสิทธิ์ Accessibility…",#selector(openPrivacy)); menu.addItem(.separator()); add(menu,"ออก",#selector(quit)); status.menu = menu; menuWillOpen(menu)
    }
    private func add(_ menu: NSMenu, _ title: String, _ action: Selector) { let i = NSMenuItem(title: title, action: action, keyEquivalent: ""); i.target = self; menu.addItem(i) }
    func menuWillOpen(_ menu: NSMenu) {
        let d = ISO8601DateFormatter(); d.formatOptions = [.withFullDate]; let counts = storeCounts(d.string(from: Date()))
        info.title = "วันนี้: พิมพ์ \(counts.0) คำ · แก้ให้ \(counts.1) · ย้อน \(counts.2)"
        toggle.state = (store.settings["enabled"] as? Bool ?? true) ? .on : .off
        clearItem.title = "ล้างคำที่ตั้งไม่ให้แปลง (\(tracker.never.count))"; clearItem.isEnabled = !tracker.never.isEmpty
        let resource = Bundle.main.resourceURL ?? URL(fileURLWithPath: "."); let personal = store.models(bundleResources: resource).path.contains("Application Support"); modelInfo.title = personal ? "โมเดล: ส่วนตัว" : "โมเดล: ทั่วไป"
    }
    private func storeCounts(_ day: String) -> (Int,Int,Int) { let u = store.root.appendingPathComponent("stats.json"); guard let data = try? Data(contentsOf: u), let all = try? JSONSerialization.jsonObject(with: data) as? [String:[String:Int]], let x = all[day] else { return (0,0,0) }; return ((x["runs_th"] ?? 0)+(x["runs_en"] ?? 0)+(x["runs_neutral"] ?? 0), (x["auto_typing"] ?? 0)+(x["auto_space"] ?? 0), (x["undo_backspace"] ?? 0)+(x["undo_shift"] ?? 0)) }
    private func askTrust() { let trusted = AXIsProcessTrustedWithOptions([kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary); if trusted { isTrusted = true; trustedTimer?.invalidate(); _ = tap.install(); refreshStatusTitle() } else { trustedTimer = Timer.scheduledTimer(withTimeInterval: 2, repeats: true) { [weak self] _ in guard let self, AXIsProcessTrusted() else { return }; self.isTrusted = true; _ = self.tap.install(); self.refreshStatusTitle(); self.trustedTimer?.invalidate() } } }
    private func refreshStatusTitle() { status.button?.title = isTrusted && engineReady ? "ก⇄A" : "ก⇄A ⚠" }
    private func record(_ event: TrackerEvent) {
        switch event {
        case .runEnded(let l): store.increment("runs_\(l)")
        case .converted(let kind, let from, let to, let score, let key, let prev, let word, let result): store.increment("auto_\(kind)"); store.appendEvent(["ts": ISO8601DateFormatter().string(from: Date()),"app": NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "","kind":kind,"from":from,"to":to,"score":score,"key":key as Any? ?? NSNull(),"prev":prev?.rawValue as Any? ?? NSNull(),"word":word,"result":result])
        case .reverted(let kind, let from, let to, let word, let result): store.increment("undo_\(kind)"); store.appendEvent(["ts":ISO8601DateFormatter().string(from: Date()),"app":NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "","kind":"undo_\(kind)","from":from,"to":to,"score":0,"key":NSNull(),"prev":NSNull(),"word":word,"result":result]); for word in tracker.never.subtracting(storedNever) { store.addNever(word) }; storedNever = tracker.never
        }
    }
    @objc private func toggleEnabled() { let value = !(store.settings["enabled"] as? Bool ?? true); store.settings["enabled"] = value; store.saveSettings(); tap.enabled = value; if !value { _ = tracker.handle(.reset) }; menuWillOpen(status.menu!) }
    @objc private func clearNever() { tracker.clearNever(); store.clearNever(); storedNever = [] }
    @objc private func showReport() {
        let resources = Bundle.main.resourceURL ?? URL(fileURLWithPath: ".")
        let path = store.models(bundleResources: resources).path
        let kind = path.contains("Application Support") ? "personal" : (path.contains("/generic") ? "generic" : "open")
        reportWindow = ReportWindowController(store: store, model: kind); reportWindow?.showWindow()
    }
    @objc private func openData() { NSWorkspace.shared.open(store.root) }
    @objc private func openPrivacy() { NSWorkspace.shared.open(URL(string:"x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!) }
    @objc private func quit() { store.flush(); NSApp.terminate(nil) }
    @objc private func appChanged() { _ = tracker.handle(.reset) }
    @objc private func inputChanged() { tap.sourceChangedByUser() }
    func applicationWillTerminate(_ notification: Notification) { store.flush() }
}

private final class ReportWindowController: NSObject, NSTextViewDelegate, NSTextFieldDelegate {
    private let store: Store
    private let model: String
    private let window: NSWindow
    private let message = NSTextView(frame: .zero)
    private let contact = NSTextField()
    private let attach = NSButton(checkboxWithTitle: "แนบคำที่แอปแปลงล่าสุด (ไม่เกิน 10 คำ)", target: nil, action: nil)
    private let preview = NSTextView(frame: .zero)
    private let send = NSButton(title: "ส่ง", target: nil, action: nil)
    private let status = NSTextField(labelWithString: "")
    init(store: Store, model: String) {
        self.store = store; self.model = model
        window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 520, height: 650), styleMask: [.titled, .closable], backing: .buffered, defer: false)
        super.init()
        window.title = "แจ้งปัญหา"
        let root = NSStackView(); root.orientation = .vertical; root.alignment = .leading; root.spacing = 8; root.translatesAutoresizingMaskIntoConstraints = false
        let content = NSView(); content.addSubview(root); NSLayoutConstraint.activate([root.leadingAnchor.constraint(equalTo: content.leadingAnchor, constant: 18), root.trailingAnchor.constraint(equalTo: content.trailingAnchor, constant: -18), root.topAnchor.constraint(equalTo: content.topAnchor, constant: 16), root.bottomAnchor.constraint(equalTo: content.bottomAnchor, constant: -16)])
        window.contentView = content
        let messageLabel = NSTextField(labelWithString: "เกิดอะไรขึ้น (ไม่เกิน 2000 ตัวอักษร)")
        message.isRichText = false; message.font = .systemFont(ofSize: 13); message.delegate = self
        let messageScroll = NSScrollView(); messageScroll.hasVerticalScroller = true; messageScroll.documentView = message; messageScroll.heightAnchor.constraint(equalToConstant: 100).isActive = true
        let contactLabel = NSTextField(labelWithString: "อีเมลหรือ LINE ID สำหรับติดต่อกลับ")
        contact.placeholderString = "ไม่บังคับ"
        contact.delegate = self
        attach.target = self; attach.action = #selector(changed)
        let previewLabel = NSTextField(labelWithString: "ตัวอย่าง JSON ที่จะส่ง")
        preview.isEditable = false; preview.isRichText = false; preview.font = .monospacedSystemFont(ofSize: 11, weight: .regular)
        let previewScroll = NSScrollView(); previewScroll.hasVerticalScroller = true; previewScroll.documentView = preview; previewScroll.heightAnchor.constraint(equalToConstant: 250).isActive = true
        send.target = self; send.action = #selector(sendReport); send.isEnabled = false
        let cancel = NSButton(title: "ยกเลิก", target: self, action: #selector(close)); let buttons = NSStackView(views: [cancel, send]); buttons.orientation = .horizontal
        root.addArrangedSubview(messageLabel); root.addArrangedSubview(messageScroll); root.addArrangedSubview(contactLabel); root.addArrangedSubview(contact); root.addArrangedSubview(attach); root.addArrangedSubview(previewLabel); root.addArrangedSubview(previewScroll); root.addArrangedSubview(status); root.addArrangedSubview(buttons)
        contact.widthAnchor.constraint(equalTo: root.widthAnchor).isActive = true
        refresh()
    }
    func showWindow() { window.center(); window.makeKeyAndOrderFront(nil); NSApp.activate(ignoringOtherApps: true) }
    func textDidChange(_ notification: Notification) { refresh() }
    func controlTextDidChange(_ notification: Notification) { refresh() }
    @objc private func changed() { refresh() }
    private func report() throws -> [String: Any] {
        let formatter = ISO8601DateFormatter(); formatter.formatOptions = [.withFullDate]
        let stats = try? Data(contentsOf: store.root.appendingPathComponent("stats.json")); let events = try? Data(contentsOf: store.root.appendingPathComponent("events.jsonl"))
        let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.2.0"
        return try ReportBuilder.build(message: message.string, contact: contact.stringValue, includeRecentWords: attach.state == .on, version: version, osVersion: ProcessInfo.processInfo.operatingSystemVersionString, model: model, enabled: store.settings["enabled"] as? Bool ?? true, statsData: stats, eventsData: events, day: formatter.string(from: Date()))
    }
    private func refresh() {
        let valid = !message.string.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty && message.string.count <= 2000
        send.isEnabled = valid
        do { preview.string = String(data: try ReportBuilder.jsonData(report()), encoding: .utf8) ?? "" }
        catch { preview.string = "ข้อความต้องมี 1–2000 ตัวอักษร" }
    }
    @objc private func close() { window.close() }
    @objc private func sendReport() {
        guard let report = try? report(), let body = try? ReportBuilder.jsonData(report) else { status.stringValue = "ข้อความไม่ถูกต้อง"; return }
        send.isEnabled = false; status.stringValue = "กำลังส่ง…"
        let urlString = store.settings["reportURL"] as? String ?? "https://keyflip.sparkth.io/v1/reports"
        guard let url = URL(string: urlString) else { status.stringValue = "URL ไม่ถูกต้อง"; send.isEnabled = true; return }
        var request = URLRequest(url: url, timeoutInterval: 10); request.httpMethod = "POST"; request.setValue("application/json", forHTTPHeaderField: "Content-Type"); request.httpBody = body
        URLSession.shared.dataTask(with: request) { [weak self] _, response, error in
            DispatchQueue.main.async {
                guard let self else { return }
                if let error { self.status.stringValue = "ส่งไม่สำเร็จ: \(error.localizedDescription)" }
                else if (response as? HTTPURLResponse)?.statusCode == 201 { self.status.stringValue = "ส่งแล้ว ขอบคุณครับ" }
                else if (response as? HTTPURLResponse)?.statusCode == 429 { self.status.stringValue = "ส่งถี่เกินไป ลองใหม่ภายหลัง" }
                else { self.status.stringValue = "ส่งไม่สำเร็จ กรุณาลองใหม่ภายหลัง" }
                self.send.isEnabled = !self.message.string.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            }
        }.resume()
    }
}
