typedef long _off_t;
typedef short __dev_t;
typedef unsigned short __uid_t;
typedef unsigned short __gid_t;
typedef int _ssize_t;

typedef int _lock_t;
typedef _lock_t _LOCK_RECURSIVE_T;
typedef _lock_t _LOCK_T;
typedef struct _reent *_data;

#include <freertos/FreeRTOS.h>
#include <freertos/task.h>
#include <freertos/timers.h>
#include <freertos/event_groups.h>

#include <esp_system.h>
#include <esp_wifi.h>
#include <esp_event.h>
#include <esp_event_loop.h>
#include <esp_log.h>

#include <nvs_flash.h>

#include <driver/uart.h>
#include <esp_http_client.h>

#include <lwip/err.h>
#include <lwip/sys.h>

#include <esp_wifi_types.h>

#include <esp_tls.h>
#include <sntp.h>
