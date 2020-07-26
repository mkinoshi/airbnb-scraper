.PHONY: run
run:
	(geckodriver --host 0.0.0.0 &) && cargo run > /tmp/log

push-image:
	docker tag scraper:latest gcr.io/airnotify/scraper:latest
	gcloud docker -- push gcr.io/airnotify/scraper:latest

setup-instance:
	gcloud compute instances create-with-container airnotify \
	      --project=airnotify \
	      --network default \
	      --zone us-west1-b \
	      --labels app=airnotify \
	      --container-image gcr.io/airnotify/scraper:latest \
	      --container-restart-policy on-failure \
	      --container-privileged

create-pub-sub:
	gcloud pubsub topics create start-instance-event --project=airnotify
	gcloud pubsub topics create stop-instance-event --project=airnotify

create-function:
	cd js_packages/scheduleinstance && gcloud functions deploy startInstancePubSub \
	  --project=airnotify \
	  --trigger-topic start-instance-event \
	  --runtime nodejs8
	cd js_packages/scheduleinstance && gcloud functions deploy stopInstancePubSub \
	  --project=airnotify \
	  --trigger-topic stop-instance-event \
	  --runtime nodejs8

stop-instance:
	gcloud functions call stopInstancePubSub --project=airnotify \
	  --data '{"data":"eyJ6b25lIjoidXMtd2VzdDEtYiIsICJsYWJlbCI6ImFwcD1haXJub3RpZnkifQo="}'

start-instance:
	gcloud functions call startInstancePubSub --project=airnotify \
	  --data '{"data":"eyJ6b25lIjoidXMtd2VzdDEtYiIsICJsYWJlbCI6ImFwcD1haXJub3RpZnkifQo="}'

create-start-cron-job:
	gcloud beta scheduler jobs create pubsub startup-instances \
	  --project=airnotify \
	  --schedule '* 0,3,6,9,12,15,18,21 * * *' \
	  --topic start-instance-event \
	  --message-body '{"zone":"us-west1-b", "label":"app=airnotify"}' \
	  --time-zone 'America/Los_Angeles'

create-stop-cron-job:
	gcloud beta scheduler jobs create pubsub stop-instances \
	  --project=airnotify \
	  --schedule '* 1,4,7,10,13,16,19,22 * * *' \
	  --topic stop-instance-event \
	  --message-body '{"zone":"us-west1-b", "label":"app=airnotify"}' \
	  --time-zone 'America/Los_Angeles'

connect-to-instance:
	gcloud compute ssh airnotify --project=airnotify
