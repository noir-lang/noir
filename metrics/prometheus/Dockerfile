FROM prom/prometheus
COPY ./prometheus.yml /etc/prometheus/prometheus.yml
USER root
CMD ["--config.file=/etc/prometheus/prometheus.yml", "--storage.tsdb.path=/prometheus", "--web.console.libraries=/usr/share/prometheus/console_libraries", "--web.console.templates=/usr/share/prometheus/consoles", "--storage.tsdb.retention.time=2y"]