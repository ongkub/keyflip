import unittest
from detect import Detector
from keymap import to_thai, to_english


class KeymapTests(unittest.TestCase):
    def test_round_trip_expected(self):
        self.assertEqual(to_thai("vpkdwxdbo-hk;"), "อยากไปกินข้าว")

    def test_round_trip_known(self):
        value = "meeting"
        self.assertEqual(to_english(to_thai(value)), value)


class DetectorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.det = Detector({"threshold": 0.0, "orthography_convert": 4.0, "orthography_keep": 4.0})

    def test_cases(self):
        self.assertEqual(self.det.decide("w,j", "th")[0], "convert")
        self.assertEqual(self.det.decide("้ำสสน", "th")[0], "convert")
        self.assertEqual(self.det.decide("hello", None)[0], "keep")
        self.assertEqual(self.det.decide("ประชุม", "en")[0], "keep")
        self.assertEqual(self.det.decide("SEO", "th")[0], "keep")
        self.assertEqual(self.det.decide("8iy[", "th")[0], "convert")

    def test_prefix_and_complete_modes(self):
        default = Detector()
        self.assertEqual(default.decide("ทั", None)[0], "keep")
        self.assertEqual(default.decide("ทั้งหมด", "th", complete=True)[0], "keep")
        self.assertEqual(default.decide("ๆ", "th")[0], "keep")
        self.assertEqual(default.decide("my", "en", complete=True)[0], "keep")
        self.assertEqual(default.decide("w,j", "th", complete=True)[0], "convert")


class FileExtensionTests(unittest.TestCase):
    def test_extensions_protected_thai_sara_ai_not(self):
        from detect import is_file_extension
        for run in (".pdf", ".PDF", ".docx", ".doc", ".jpe", ".mp4"):
            self.assertTrue(is_file_extension(run), run)
        for run in (".o", ".s'h", ".0", ".8i", ".s,j", ".p", "pdf", ".pdfx"):
            self.assertFalse(is_file_extension(run), run)


if __name__ == "__main__": unittest.main()


