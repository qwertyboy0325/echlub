# Phones as terminals

> **SIMULATION.** Assumed phone audio and network profiles. `wired` for a phone means a USB-C Ethernet adapter. Forward topology, 2 copies per frame, 95% jitter coverage.

| Case | Players | Mouth-to-ear p50 range (ms) | Tempo drift | Playable+ | Worst audible dropouts/min |
| --- | --- | --- | --- | --- | --- |
| baseline-all-laptop-interface | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/fiber-wired` | 15.0–17.7 | -0.79% | 100% | 0.0 |
| vocals-phone-interface-wired | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/fiber-wired/phone-interface` | 15.0–19.5 | -0.84% | 100% | 0.0 |
| vocals-phone-interface-wifi | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/wifi/phone-interface` | 15.0–27.5 | -1.10% | 100% | 0.0 |
| vocals-phone-interface-5g-sa | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/mobile-5g-sa/phone-interface` | 15.0–27.5 | -1.10% | 100% | 0.0 |
| vocals-phone-ios-app-wired | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/fiber-wired/phone-ios-app` | 15.0–22.7 | -0.94% | 100% | 0.0 |
| vocals-phone-ios-app-wifi | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/wifi/phone-ios-app` | 15.0–30.7 | -1.20% | 100% | 0.0 |
| vocals-phone-ios-app-5g-sa | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/mobile-5g-sa/phone-ios-app` | 15.0–30.7 | -1.20% | 100% | 0.0 |
| vocals-phone-android-low-latency-wired | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/fiber-wired/phone-android-low-latency` | 15.0–27.3 | -1.01% | 100% | 0.0 |
| vocals-phone-android-low-latency-wifi | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/wifi/phone-android-low-latency` | 15.0–35.3 | -1.27% | 100% | 0.0 |
| vocals-phone-android-low-latency-5g-sa | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/mobile-5g-sa/phone-android-low-latency` | 15.0–35.3 | -1.27% | 100% | 0.0 |
| vocals-phone-android-generic-wired | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/fiber-wired/phone-android-generic` | 15.0–47.8 | -1.56% | 95% | 0.0 |
| vocals-phone-android-generic-wifi | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/wifi/phone-android-generic` | 15.0–55.8 | -1.82% | 0% | 0.0 |
| vocals-phone-android-generic-5g-sa | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/mobile-5g-sa/phone-android-generic` | 15.0–55.8 | -1.82% | 0% | 0.0 |
| vocals-phone-browser-wired | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/fiber-wired/phone-browser` | 15.0–48.8 | -1.57% | 95% | 0.0 |
| vocals-phone-browser-wifi | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/wifi/phone-browser` | 15.0–56.8 | -1.83% | 0% | 0.0 |
| vocals-phone-browser-5g-sa | `drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/mobile-5g-sa/phone-browser` | 15.0–56.8 | -1.83% | 0% | 0.0 |
| all4-phone-ios-app-wired | `drums@taipei/fiber-wired/phone-ios-app,bass@taichung/fiber-wired/phone-ios-app,guitar@tainan/fiber-wired/phone-ios-app,vocals@kaohsiung/fiber-wired/phone-ios-app` | 24.5–27.2 | -1.76% | 100% | 0.0 |
| all4-phone-ios-app-wifi | `drums@taipei/wifi/phone-ios-app,bass@taichung/wifi/phone-ios-app,guitar@tainan/wifi/phone-ios-app,vocals@kaohsiung/wifi/phone-ios-app` | 35.2–43.2 | -3.11% | 0% | 0.0 |
| all4-phone-android-low-latency-wired | `drums@taipei/fiber-wired/phone-android-low-latency,bass@taichung/fiber-wired/phone-android-low-latency,guitar@tainan/fiber-wired/phone-android-low-latency,vocals@kaohsiung/fiber-wired/phone-android-low-latency` | 28.8–31.5 | -2.20% | 100% | 0.0 |
| all4-phone-android-low-latency-wifi | `drums@taipei/wifi/phone-android-low-latency,bass@taichung/wifi/phone-android-low-latency,guitar@tainan/wifi/phone-android-low-latency,vocals@kaohsiung/wifi/phone-android-low-latency` | 39.5–47.5 | -3.53% | 0% | 0.0 |
| all4-phone-android-generic-wired | `drums@taipei/fiber-wired/phone-android-generic,bass@taichung/fiber-wired/phone-android-generic,guitar@tainan/fiber-wired/phone-android-generic,vocals@kaohsiung/fiber-wired/phone-android-generic` | 64.3–67.0 | -5.62% | 0% | 0.0 |
| all4-phone-android-generic-wifi | `drums@taipei/wifi/phone-android-generic,bass@taichung/wifi/phone-android-generic,guitar@tainan/wifi/phone-android-generic,vocals@kaohsiung/wifi/phone-android-generic` | 75.0–83.0 | -6.87% | 0% | 0.0 |
