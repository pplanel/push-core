from gevent import sleep
from locust import task, between
from websocket_locust_user import WebsocketUser
import logging


class PushCoreUser(WebsocketUser):
    fixed_count = 10
    wait_time = between(1, 5)

    @task
    def my_task(self):
        self.connect("ws://localhost:3000/ws")

        while True:
            self.send("Hello")
            sleep(100)

    def on_message(self, message):
        logging.debug(f"Message received {message}")
