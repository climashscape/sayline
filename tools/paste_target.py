#!/usr/bin/env python3
"""Disposable X11 target window for verifying auto-paste."""
import tkinter as tk


def main():
    root = tk.Tk()
    root.title("Sayline Paste Target")
    root.geometry("620x140+60+60")
    entry = tk.Entry(root, width=70, font=("Sans", 13))
    entry.pack(padx=18, pady=24, fill="x")
    entry.focus_set()
    root.lift()
    root.focus_force()

    def keep_focus():
        entry.focus_force()
        root.after(500, keep_focus)

    root.after(200, keep_focus)

    def report():
        print("ENTRY=" + entry.get(), flush=True)
        root.destroy()

    def report_focus():
        print(f"FOCUS={root.focus_get()}", flush=True)
        root.after(10000, report)

    root.after(5000, report_focus)
    root.after(9000, report)
    root.mainloop()


if __name__ == "__main__":
    main()
