jni := "android/app/src/main/jniLibs"
abis := "-t arm64-v8a -t armeabi-v7a -t x86 -t x86_64"

# Debug build for one ABI (default: emulator), install and launch. e.g. `just android arm64-v8a`
android abi="x86_64":
    rm -rf {{jni}}
    cargo ndk -t {{abi}} -P 26 -o {{jni}} build --lib
    rm -f {{jni}}/*/libgpui_mobile-*.so
    cd android && ./gradlew installDebug
    adb shell am start -n io.profidev.lt/dev.gpui.mobile.GpuiActivity

# Release APK + AAB for all ABIs, signed when android/keystore.properties exists
android-release:
    rm -rf {{jni}}
    cargo ndk {{abis}} -P 26 -o {{jni}} build --lib --release
    rm -f {{jni}}/*/libgpui_mobile-*.so
    cd android && ./gradlew assembleRelease bundleRelease

# macOS only: generate the Xcode project (its pre-build phase runs cargo)
ios:
    cd ios && xcodegen generate --spec project.yml
    cd ios && xcodebuild -project Lt.xcodeproj -scheme Lt -sdk iphonesimulator -configuration Debug -derivedDataPath build build

# Desktop app (Linux / macOS / Windows)
desktop:
    cargo run
