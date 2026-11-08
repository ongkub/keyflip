#!/usr/bin/env python3
"""สร้าง Quick Action "สลับภาษา ไทย-อังกฤษ" จาก convert.js แล้วติดตั้งที่ ~/Library/Services

ใช้: python3 build.py            ติดตั้ง/อัปเดต
     python3 build.py --out DIR  สร้างไว้ที่ DIR แทน (ไม่ติดตั้ง)
ตัว Quick Action ใช้แค่ zsh + osascript ที่มากับ macOS ไม่ต้องมี Python ตอนใช้งาน
"""
import os
import plistlib
import shutil
import sys
import uuid

NAME = "สลับภาษา ไทย-อังกฤษ"
HERE = os.path.dirname(os.path.abspath(__file__))
TEXT_UTI = "public.utf8-plain-text"


def uid():
    return str(uuid.uuid4()).upper()


def command_string(js):
    assert "\nKEYFLIP_JS\n" not in js
    return (
        "script=$(cat <<'KEYFLIP_JS'\n" + js.rstrip("\n") + "\nKEYFLIP_JS\n)\n"
        'osascript -l JavaScript -e "$script"\n'
    )


def wflow(js):
    args = [
        ("inputMethod", 0),
        ("CheckedForUserDefaultShell", False),
        ("source", ""),
        ("COMMAND_STRING", ""),
        ("shell", "/bin/sh"),
    ]
    action = {
        "AMAccepts": {"Container": "List", "Optional": True, "Types": ["com.apple.cocoa.string"]},
        "AMActionVersion": "2.0.3",
        "AMApplication": ["Automator"],
        "AMParameterProperties": {k: {} for k, _ in args},
        "AMProvides": {"Container": "List", "Types": ["com.apple.cocoa.string"]},
        "ActionBundlePath": "/System/Library/Automator/Run Shell Script.action",
        "ActionName": "Run Shell Script",
        "ActionParameters": {
            "COMMAND_STRING": command_string(js),
            "CheckedForUserDefaultShell": True,
            "inputMethod": 0,  # ส่งข้อความที่เลือกเข้า stdin
            "shell": "/bin/zsh",
            "source": "",
        },
        "BundleIdentifier": "com.apple.RunShellScript",
        "CFBundleVersion": "2.0.3",
        "CanShowSelectedItemsWhenRun": False,
        "CanShowWhenRun": True,
        "Category": ["AMCategoryUtilities"],
        "Class Name": "RunShellScriptAction",
        "InputUUID": uid(),
        "Keywords": ["Shell", "Script", "Command", "Run", "Unix"],
        "OutputUUID": uid(),
        "UUID": uid(),
        "UnlocalizedApplications": ["Automator"],
        "arguments": {
            str(i): {"default value": d, "name": n, "required": "0", "type": "0", "uuid": str(i)}
            for i, (n, d) in enumerate(args)
        },
        "conversionLabel": 0,
        "isViewVisible": 1,
        "location": "309.000000:305.000000",
        "nibPath": "/System/Library/Automator/Run Shell Script.action/Contents/Resources/Base.lproj/main.nib",
    }
    return {
        "AMApplicationBuild": "534",
        "AMApplicationVersion": "2.10",
        "AMDocumentVersion": "2",
        "actions": [{"action": action, "isViewVisible": 1}],
        "connectors": {},
        "workflowMetaData": {
            "applicationBundleIDsByPath": {},
            "applicationPaths": [],
            "inputTypeIdentifier": "com.apple.Automator.text",
            "outputTypeIdentifier": "com.apple.Automator.text",
            "presentationMode": 11,
            "processesInput": False,
            "serviceInputTypeIdentifier": "com.apple.Automator.text",
            "serviceOutputTypeIdentifier": "com.apple.Automator.text",  # ผลลัพธ์แทนที่ข้อความที่เลือก
            "serviceProcessesInput": False,
            "systemImageName": "NSActionTemplate",
            "useAutomaticInputType": False,
            "workflowTypeIdentifier": "com.apple.Automator.servicesMenu",
        },
    }


def info_plist():
    return {
        "NSServices": [
            {
                "NSBackgroundColorName": "background",
                "NSIconName": "NSActionTemplate",
                "NSMenuItem": {"default": NAME},
                "NSMessage": "runWorkflowAsService",
                "NSSendTypes": [TEXT_UTI],
                "NSReturnTypes": [TEXT_UTI],
            }
        ]
    }


def main():
    out = os.path.expanduser("~/Library/Services")
    if "--out" in sys.argv:
        out = sys.argv[sys.argv.index("--out") + 1]
    js = open(os.path.join(HERE, "convert.js"), encoding="utf-8").read()
    target = os.path.join(out, NAME + ".workflow")
    if os.path.exists(target):
        shutil.rmtree(target)
    os.makedirs(os.path.join(target, "Contents"))
    with open(os.path.join(target, "Contents", "Info.plist"), "wb") as f:
        plistlib.dump(info_plist(), f)
    with open(os.path.join(target, "Contents", "document.wflow"), "wb") as f:
        plistlib.dump(wflow(js), f)
    print(target)


if __name__ == "__main__":
    main()
