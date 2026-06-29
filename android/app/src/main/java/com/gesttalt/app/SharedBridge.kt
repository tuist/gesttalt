package com.gesttalt.app

object SharedBridge {
    init {
        System.loadLibrary("shared")
    }

    fun greeting(name: String): String {
        val trimmed = name.trim()
        val displayName = if (trimmed.isEmpty()) "there" else trimmed
        return "Hello, $displayName. This score came from shared Rust."
    }

    @JvmStatic
    external fun latticeScore(seed: Int): Int
}
