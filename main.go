package main

import (
	"context"
	"errors"
	"flag"
	"net/http"
	"os"
	"os/signal"
	"strconv"
	"syscall"
	"time"

	"github.com/ForbiddenR/redis_exporter/exporter"
	"github.com/ForbiddenR/redis_exporter/log"
	"github.com/prometheus/client_golang/prometheus"
)

func getEnv(key string, defaultVal string) string {
	if value, exists := os.LookupEnv(key); exists {
		return value
	}
	return defaultVal
}

func getEnvBool(key string, defaultVal bool) bool {
	if envVal, ok := os.LookupEnv(key); ok {
		envBool, err := strconv.ParseBool(envVal)
		if err == nil {
			return envBool
		}
	}
	return defaultVal
}

var (
	redisAddr                    = flag.String("redis.addr", getEnv("REDIS_ADDR", "localhost:26379"), "Redis address")
	redisUser                    = flag.String("redis.user", getEnv("REDIS_USER", ""), "Redis username")
	redisPwd                     = flag.String("redis.password", getEnv("REDIS_PASSWORD", ""), "Redis password")
	namespace                    = flag.String("namespace", getEnv("REDIS_EXPORTER_NAMESPACE", "redis"), "Prometheus namespace")
	listenAddress                = flag.String("web.listen-address", getEnv("REDIS_EXPORTER_WEB_LISTEN_ADDRESS", ":9999"), "Address to listen on for web interface and telemetry.")
	inclMetricsForEmptyDatabases = flag.Bool("include-metrics-for-empty-databases", getEnvBool("REDIS_EXPORTER_INCL_METRICS_FOR_EMPTY_DATABASES", true), "Whether to emit db metrics (like db_keys) for empty databases")
)

func main() {
	flag.Parse()

	registry := prometheus.NewRegistry()

	exp, err := exporter.NewRedisExporter(*redisAddr, exporter.Options{
		User:                         *redisUser,
		Password:                     *redisPwd,
		Namespace:                    *namespace,
		InclMetricsForEmptyDatabases: *inclMetricsForEmptyDatabases,
		Registry:                     registry,
	})
	if err != nil {
		panic(err)
	}

	server := &http.Server{
		Addr:    *listenAddress,
		Handler: exp,
	}
	go func() {
		if err := server.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			log.Fatalf("Server error: %v", err)
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	_quit := <-quit
	log.Infof("Received signal: %s, shutting down...", _quit.String())
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if err := server.Shutdown(ctx); err != nil {
		log.Fatalf("Server shutdown failed: %v", err)
	}
	log.Infof("Server gracefully stopped")
}
