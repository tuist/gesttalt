package com.gesttalt.app

object SharedBridge {
    init {
        System.loadLibrary("shared")
    }

    @JvmStatic
    external fun greeting(name: String): String

    @JvmStatic
    external fun latticeScore(seed: Int): Int
}

