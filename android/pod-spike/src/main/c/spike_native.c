#include <jni.h>

JNIEXPORT jint JNICALL Java_dev_zlauncher_podspike_SpikeNative_getMagicNumber(JNIEnv *env, jclass clazz) {
    return 1337;
}
