package exporter

import (
	"fmt"
	"strconv"

	"github.com/ForbiddenR/redis_exporter/log"
	"github.com/prometheus/client_golang/prometheus"
)

func newMetricDescr(namespace, metricName, docString string, labels ...string) *prometheus.Desc {
	return prometheus.NewDesc(prometheus.BuildFQName(namespace, "", metricName), docString, labels, nil)
}

func (e *Exporter) includeMetric(s string) bool {
	_, ok := e.metricMapGauges[s]
	return ok
}

func (e *Exporter) parseAndRegisterConstMetric(ch chan<- prometheus.Metric, fieldKey, fieldValue string) {
	metricName := fieldKey

	// if newName, ok := e.metricMapGauges[metricName]; ok {
	// 	metricName = newName
	// }

	var err error
	var val float64

	switch fieldValue {
	case "ok", "true":
		val = 1
	case "err", "fail", "false":
		val = 0
	default:
		val, err = strconv.ParseFloat(fieldValue, 64)
	}

	if err != nil {
		log.Infof("couldn't parse %s, err: %s", fieldValue, err)
		return
	}

	t := prometheus.GaugeValue

	e.registerConstMetric(ch, metricName, val, t)
}

func (e *Exporter) registerConstMetricGauge(ch chan<- prometheus.Metric, metric string, val float64, labelValues ...string) {
	e.registerConstMetric(ch, metric, val, prometheus.GaugeValue, labelValues...)
}

func (e *Exporter) registerConstMetric(ch chan<- prometheus.Metric, metric string, val float64, valType prometheus.ValueType, labelValues ...string) {
	var desc *prometheus.Desc
	if len(labelValues) == 0 {
		desc = e.createMetricDescription(metric)
	} else {
		desc = e.mustFindMetricDescription(metric)
	}

	m, err := prometheus.NewConstMetric(desc, valType, val, labelValues...)
	if err != nil {
		fmt.Printf("registerConstMetric( %s, %.2f) err: %s\n", metric, val, err)
		return
	}

	ch <- m
}

func (e *Exporter) mustFindMetricDescription(metricName string) *prometheus.Desc {
	desc, ok := e.metricDescriptons[metricName]
	if !ok {
		panic(fmt.Sprintf("couldn't find metric description for %s", metricName))
	}
	return desc
}

func (e *Exporter) createMetricDescription(metricName string, labels ...string) *prometheus.Desc {
	if desc, ok := e.metricDescriptons[metricName]; ok {
		return desc
	}

	desc := newMetricDescr(e.options.Namespace, metricName, metricName+" metric", labels...)
	e.metricDescriptons[metricName] = desc

	return desc
}
