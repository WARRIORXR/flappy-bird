# 📱 Flappy Bird — Android App & APK Build Guide

This directory contains the complete **native Android Studio project** for Flappy Bird Pro.

---

## ⚡ Option 1: Play Immediately on Android (Zero Installation / PWA)

You don't even need to build an APK to play on your phone!

1. Start the server on your computer:
   ```bash
   npm start
   ```
2. Open Chrome or Firefox on your Android device.
3. Navigate to the network URL displayed in the terminal (e.g. `http://192.168.x.x:3030`).
4. Tap the **"📱 Install App on Android"** banner or tap Chrome's three dots menu (`⋮`) and select **"Install App"** (or **"Add to Home screen"**).
5. The game installs instantly as a full-screen, standalone Android app with:
   - ⚡ Offline play capability (powered by Service Worker)
   - 📳 Haptic touch vibrations
   - 🖼️ Custom launcher icon & splash screen
   - 🎮 Smooth 60 FPS hardware-accelerated WebGL performance

---

## 📦 Option 2: Build Native Android APK (`.apk`)

### With Android Studio:
1. Open **Android Studio**.
2. Select **File > Open...** and choose the `android` folder in this directory:
   ```
   d:\ANTIGRAVITY\flappy bird\android
   ```
3. Wait for Gradle sync to complete.
4. Click **Build > Build Bundle(s) / APK(s) > Build APK(s)**.
5. Once compiled, click **locate** to find your release `.apk` in:
   `android/app/build/outputs/apk/debug/app-debug.apk`
6. Transfer this APK to your Android device and install it!

### With Command-Line Gradle:
```bash
cd android
./gradlew assembleDebug
# Generated APK at: app/build/outputs/apk/debug/app-debug.apk
```

---

## 🛠️ Architecture Highlights:
- **Offline Assets**: All web assets, canvas shaders, sound synthesizers, and WebAssembly binaries are pre-bundled in `app/src/main/assets/`.
- **Immersive Sticky Fullscreen**: Automatically hides the Android status bar and navigation gestures.
- **Hardware Acceleration**: GPU canvas acceleration enabled via `android:hardwareAccelerated="true"`.
- **Haptic Bridge**: Direct native Android `Vibrator` hardware integration via JavaScript interface.
