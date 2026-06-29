package com.gesttalt.app

import android.app.Activity
import android.graphics.Typeface
import android.os.Bundle
import android.view.Gravity
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView

class MainActivity : Activity() {
    private var counter = 0

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val greeting = TextView(this).apply {
            text = SharedBridge.greeting("Kotlin")
            textSize = 22f
            typeface = Typeface.DEFAULT_BOLD
        }
        val score = TextView(this).apply {
            text = scoreText("Kotlin")
            textSize = 16f
        }
        val button = Button(this).apply {
            text = "Refresh"
            setOnClickListener {
                counter += 1
                val name = "Kotlin $counter"
                greeting.text = SharedBridge.greeting(name)
                score.text = scoreText(name)
            }
        }

        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(48, 48, 48, 48)
            addView(greeting, matchWidth())
            addView(score, matchWidth())
            addView(button, wrapContent())
        }

        setContentView(root)
    }

    private fun scoreText(name: String): String {
        return "Lattice score: ${SharedBridge.latticeScore(name.length)}"
    }

    private fun matchWidth(): LinearLayout.LayoutParams {
        return LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            ViewGroup.LayoutParams.WRAP_CONTENT,
        ).apply {
            bottomMargin = 24
        }
    }

    private fun wrapContent(): LinearLayout.LayoutParams {
        return LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.WRAP_CONTENT,
            ViewGroup.LayoutParams.WRAP_CONTENT,
        )
    }
}

