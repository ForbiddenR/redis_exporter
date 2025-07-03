package exporter

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/ForbiddenR/redis_exporter/log"
	"github.com/prometheus/client_golang/prometheus"
)

func extractVal(s string) (val float64, err error) {
	split := strings.Split(s, "=")
	if len(split) != 2 {
		return 0, fmt.Errorf("nope")
	}
	val, err = strconv.ParseFloat(split[1], 64)
	if err != nil {
		return 0, fmt.Errorf("nope")
	}
	return
}

func (e *Exporter) extractInfoMetrics(ch chan<- prometheus.Metric, info string, dbCount int) string {
	keyValues := map[string]string{}
	handleDBs := map[string]bool{}

	fieldClass := ""
	lines := strings.SplitSeq(info, "\n")
	for line := range lines {
		line = strings.TrimSpace(line)
		log.Debugf("info: %s", line)
		if len(line) > 0 && strings.HasPrefix(line, "# ") {
			fieldClass = line[2:]
			log.Debugf("set fieldClass: %s", fieldClass)
			continue
		}

		if (len(line) < 2) || (!strings.Contains(line, ":")) {
			continue
		}

		split := strings.SplitN(line, ":", 2)
		fieldKey := split[0]
		fieldValue := split[1]

		keyValues[fieldKey] = fieldValue

		switch fieldClass {
		case "Keyspace":
			if keysTotal, keysEx, ok := parseDBKeyspaceString(fieldKey, fieldValue); ok {
				dbName := fieldKey
				e.registerConstMetricGauge(ch, "db_keys", keysTotal, dbName)
				e.registerConstMetricGauge(ch, "db_keys_expiring", keysEx, dbName)
				handleDBs[dbName] = true
				continue
			}
		}

		if !e.includeMetric(fieldKey) {
			continue
		}

		e.parseAndRegisterConstMetric(ch, fieldKey, fieldValue)
	}

	if e.options.InclMetricsForEmptyDatabases {
		for dbIndex := range dbCount {
			dbName := "db" + strconv.Itoa(dbIndex)
			if _ , exists := handleDBs[dbName]; !exists {
				e.registerConstMetricGauge(ch, "db_keys", 0, dbName)
				e.registerConstMetricGauge(ch, "db_keys_expiring", 0, dbName)
			}
		}
	}

	return keyValues["role"]
}

func parseDBKeyspaceString(inputKey, inputVal string) (keysTotal float64, keysExpiringTotal float64, ok bool) {
	log.Debugf("parseDBKeyspaceString inputKey: [%s], inputVal: [%s]", inputKey, inputVal)

	if !strings.HasPrefix(inputKey, "db") {
		log.Debugf("parseDBKeyspaceString: inputKey [%s] does not start with 'db'", inputKey)
		return
	}

	split := strings.Split(inputVal, ",")
	if len(split) < 2 || len(split) > 4 {
		log.Debugf("parseDBKeyspaceString strings.Split(inputVal) invalid: %#v", split)
		return
	}

	var err error
	if keysTotal, err = extractVal(split[0]); err != nil {
		log.Debugf("parseDBKeyspaceString extractVal(split[0]) invalid, err: %s", err)
		return
	}
	if keysExpiringTotal, err = extractVal(split[1]); err != nil {
		log.Debugf("parseDBKeyspaceString extractVal(split[1]) invalid, err: %s", err)
		return
	}

	ok = true
	return
}
