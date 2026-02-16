import { createApp } from 'vue'
import { createPinia } from 'pinia'
import vuetify from './plugins/vuetify'
import ECharts from 'vue-echarts'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { BarChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, LegendComponent } from 'echarts/components'
import App from './App.vue'

// Register ECharts components
use([CanvasRenderer, BarChart, GridComponent, TooltipComponent, LegendComponent])

const app = createApp(App)

app.use(createPinia())
app.use(vuetify)
app.component('v-chart', ECharts)

app.mount('#app')
