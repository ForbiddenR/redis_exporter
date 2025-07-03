package exporter

import (
	"sync"

	"github.com/redis/go-redis/v9"
)

var redisClient func() *redis.Client

func (e *Exporter) setRedisClient() {
	redisClient = sync.OnceValue(func() *redis.Client {
		return redis.NewClient(&redis.Options{
			Addr:     e.redisAddr,
			Username: e.options.User,
			Password: e.options.Password,
		})
	})
}

func (e *Exporter) getRedisClient() *redis.Client {
	if redisClient == nil {
		e.setRedisClient()
	}
	return redisClient()
}
