package log

import (
	"fmt"
	"log/slog"
	"os"
)

var log *slog.Logger

func init() {
	log = slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelDebug}))
}

func Info(msg string, args ...any) {
	log.Info(msg, args...)
}

func Warn(msg string, args ...any) {
	log.Warn(msg, args...)
}

func Error(msg string, args ...any) {
	log.Error(msg, args...)
}

func Debug(msg string, args ...any) {
	log.Debug(msg, args...)
}

func Fatal(msg string, args ...any) {
	log.Error(msg, args...)
	os.Exit(1)
}

func Infof(format string, args ...any) {
	Info(fmt.Sprintf(format, args...))
}

func Warnf(format string, args ...any) {
	Warn(fmt.Sprintf(format, args...))
}

func Errorf(format string, args ...any) {
	Error(fmt.Sprintf(format, args...))
}

func Debugf(format string, args ...any) {
	Debug(fmt.Sprintf(format, args...))
}

func Fatalf(format string, args ...any) {
	Fatal(fmt.Sprintf(format, args...))
}
