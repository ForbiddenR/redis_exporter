package exporter

import (
	"context"
	"fmt"
	"net/http"
	"strings"
	"sync"

	"github.com/ForbiddenR/redis_exporter/log"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

type Exporter struct {
	sync.Mutex

	redisAddr string

	metricDescriptons map[string]*prometheus.Desc

	options Options

	metricMapGauges map[string]string

	mux *http.ServeMux
}

type Options struct {
	User      string
	Password  string
	Namespace string

	Registry                     *prometheus.Registry
	InclMetricsForEmptyDatabases bool
}

func NewRedisExporter(uri string, opts Options) (*Exporter, error) {
	e := &Exporter{
		redisAddr: uri,
		options:   opts,

		metricMapGauges: map[string]string{
			"connected_clients": "connected_clients",
			"maxclients":        "max_clients",
		},
	}

	e.metricDescriptons = map[string]*prometheus.Desc{}

	for k, desc := range map[string]struct {
		txt  string
		lbls []string
	}{
		"db_keys":          {txt: "Total number of keys by DB", lbls: []string{"db"}},
		"db_keys_expiring": {txt: "Total number of expiring keys by DB", lbls: []string{"db"}},
	} {
		e.metricDescriptons[k] = newMetricDescr(opts.Namespace, k, desc.txt, desc.lbls...)
	}

	e.mux = http.NewServeMux()

	if e.options.Registry != nil {
		e.options.Registry.MustRegister(e)
		e.mux.Handle("/metrics", promhttp.HandlerFor(e.options.Registry, promhttp.HandlerOpts{}))
	}

	return e, nil
}

func (e *Exporter) Describe(ch chan<- *prometheus.Desc) {
	for _, v := range e.metricMapGauges {
		ch <- newMetricDescr(e.options.Namespace, v, v+" metric")
	}
}

func (e *Exporter) Collect(ch chan<- prometheus.Metric) {
	e.Lock()
	defer e.Unlock()

	if e.redisAddr != "" {
		if err := e.scrapeRedisHost(ch); err != nil {
			fmt.Println("Error scraping Redis host:", err)
		} else {
			fmt.Println("Successfully scraped Redis host")
		}
	}
}

func (e *Exporter) scrapeRedisHost(ch chan<- prometheus.Metric) error {
	ctx := context.TODO()
	// startTime := time.Now()

	dbCount := 0

	client := e.getRedisClient()
	infoAll := client.Info(ctx, "all").String()
	if infoAll == "" {
		infoAll = client.Info(ctx).String()
	}

	if strings.Contains(infoAll, "cluster_enabled:1") {
		if err := client.ClusterInfo(ctx).Err(); err == nil {

			// in cluster mode, Redis only supports one database
			dbCount = 1
		} else {
			log.Errorf("REDIS CLUSTER INFO err: %s", err)
		}
	} else if dbCount == 0 {
		dbCount = 16 // default Redis databases
	}

	_ = e.extractInfoMetrics(ch, infoAll, dbCount)

	return nil
}
