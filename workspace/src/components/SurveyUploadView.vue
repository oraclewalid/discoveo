<script setup lang="ts">
import { ref } from 'vue'
import Papa from 'papaparse'
import config from '@/config'

const props = defineProps<{
  projectId: string
}>()

const file = ref<File | null>(null)
const isUploading = ref(false)
const uploadStatus = ref<{ type: 'success' | 'error'; message: string } | null>(null)
const dragOver = ref(false)

const fileInput = ref<HTMLInputElement | null>(null)

const REQUIRED_HEADERS = [
  'Date',
  'Country',
  'URL',
  'Device',
  'Browser',
  'OS',
  'Comments'
]

const OPTIONAL_HEADERS = [
  'Ratings'
]

const validateCSV = (selectedFile: File) => {
  return new Promise<{ isValid: boolean; error?: string }>((resolve) => {
    Papa.parse(selectedFile, {
      preview: 1, // Read only the first row (headers)
      header: false,
      skipEmptyLines: true,
      complete: (results) => {
        if (results.errors.length > 0) {
          resolve({ isValid: false, error: 'CSV parsing error. Please check the file formatting.' })
          return
        }

        const headers = results.data[0] as string[]
        if (!headers || headers.length === 0) {
          resolve({ isValid: false, error: 'The CSV file appears to be empty.' })
          return
        }

        // Check for missing required headers
        const missingHeaders = REQUIRED_HEADERS.filter(
          (required) => !headers.some(h => String(h).trim().toLowerCase() === required.toLowerCase())
        )

        if (missingHeaders.length > 0) {
          resolve({ 
            isValid: false, 
            error: `Missing required columns: ${missingHeaders.join(', ')}` 
          })
        } else {
          resolve({ isValid: true })
        }
      },
      error: (err) => {
        resolve({ isValid: false, error: `Error reading file: ${err.message}` })
      }
    })
  })
}

const handleFileSelect = async (event: Event) => {
  const target = event.target as HTMLInputElement
  if (target.files && target.files.length > 0) {
    const selectedFile = target.files[0]
    if (selectedFile) await processFile(selectedFile)
  }
}

const handleDrop = async (event: DragEvent) => {
  dragOver.value = false
  if (event.dataTransfer?.files && event.dataTransfer.files.length > 0) {
    const droppedFile = event.dataTransfer.files[0]
    if (droppedFile) await processFile(droppedFile)
  }
}

const processFile = async (selectedFile: File) => {
  uploadStatus.value = null
  
  if (selectedFile.type !== 'text/csv' && !selectedFile.name.endsWith('.csv')) {
    uploadStatus.value = { type: 'error', message: 'Please select a valid CSV file.' }
    file.value = null
    return
  }

  const validation = await validateCSV(selectedFile)
  if (!validation.isValid) {
    uploadStatus.value = { type: 'error', message: validation.error || 'Invalid CSV format.' }
    file.value = null
  } else {
    file.value = selectedFile
  }
}

const uploadFile = async () => {
  if (!file.value) return

  isUploading.value = true
  uploadStatus.value = null

  try {
    const formData = new FormData()
    // Ensure we are appending the actual File object
    formData.append('file', file.value)

    // Prepare headers - we take everything from config but EXPLICITLY 
    // leave out Content-Type to let the browser set it with the boundary
    const headers: Record<string, string> = {}
    if (config.api.headers) {
      Object.entries(config.api.headers).forEach(([key, value]) => {
        if (key.toLowerCase() !== 'content-type') {
          headers[key] = value
        }
      })
    }

    const response = await fetch(`${config.api.baseUrl}projects/${props.projectId}/qualitative/surveys`, {
      method: 'POST',
      headers,
      body: formData,
    })

    if (!response.ok) {
      const errorData = await response.json().catch(() => null)
      const errorMessage = errorData?.error || errorData?.message || response.statusText
      throw new Error(errorMessage)
    }

    const result = await response.json()
    uploadStatus.value = { 
      type: 'success', 
      message: `Successfully uploaded ${result.row_count} survey rows!` 
    }
    file.value = null
  } catch (error) {
    console.error('Upload error:', error)
    uploadStatus.value = { 
      type: 'error', 
      message: error instanceof Error ? error.message : 'An error occurred during upload.' 
    }
  } finally {
    isUploading.value = false
  }
}

const clearFile = () => {
  file.value = null
  uploadStatus.value = null
}
</script>

<template>
  <div class="survey-upload-view">
    <div class="mb-8">
      <h1 class="text-h3 font-weight-bold tracking-tight mb-2">Upload Qualitative Data</h1>
      <p class="text-subtitle-1 text-grey-darken-1">Import your survey results via CSV file</p>
    </div>

    <v-row justify="center">
      <v-col cols="12" md="8" lg="6">
        <v-card class="premium-upload-card" elevation="0">
          <v-card-text class="pa-10">
            <div 
              class="upload-zone"
              :class="{ 'drag-over': dragOver }"
              @dragover.prevent="dragOver = true"
              @dragleave.prevent="dragOver = false"
              @drop.prevent="handleDrop"
              @click="fileInput?.click()"
            >
              <input 
                type="file" 
                ref="fileInput" 
                class="d-none" 
                accept=".csv"
                @change="handleFileSelect"
              >
              
              <v-avatar color="primary-lighten-5" size="100" class="mb-6">
                <v-icon icon="mdi-cloud-upload-outline" color="primary" size="48" />
              </v-avatar>

              <template v-if="!file">
                <h2 class="text-h5 font-weight-bold mb-2">Click or drag to upload</h2>
                <p class="text-body-1 text-grey">Support for .csv files only</p>
              </template>
              
              <template v-else>
                <div class="d-flex align-center bg-white pa-4 rounded-xl border mb-4 shadow-sm w-100">
                  <v-icon icon="mdi-file-delimited-outline" color="primary" class="mr-4" size="32" />
                  <div class="text-left flex-grow-1 overflow-hidden">
                    <div class="text-subtitle-1 font-weight-bold text-truncate">{{ file.name }}</div>
                    <div class="text-caption text-grey">{{ (file.size / 1024).toFixed(1) }} KB</div>
                  </div>
                  <v-btn icon="mdi-close" variant="text" size="small" color="grey" @click.stop="clearFile" />
                </div>
              </template>
            </div>

            <v-alert
              v-if="uploadStatus"
              :type="uploadStatus.type"
              variant="tonal"
              closable
              class="mt-6 rounded-xl"
              @click:close="uploadStatus = null"
            >
              {{ uploadStatus.message }}
            </v-alert>

            <div class="mt-8">
              <v-btn
                block
                color="primary"
                size="x-large"
                class="rounded-xl font-weight-bold elevation-4"
                :disabled="!file"
                :loading="isUploading"
                @click="uploadFile"
              >
                Upload Survey Data
              </v-btn>
            </div>
          </v-card-text>
        </v-card>

        <v-card class="mt-8 side-card" elevation="0">
          <v-card-title class="pa-6 border-bottom">
            <span class="text-h6 font-weight-bold">CSV Template Guide</span>
          </v-card-title>
          <v-card-text class="pa-6">
            <p class="text-body-2 text-grey-darken-1 mb-4">
              To ensure proper processing, please make sure your CSV includes the following headers:
            </p>
            <div class="mb-4">
              <div class="text-caption font-weight-bold text-grey mb-1">REQUIRED</div>
              <v-chip-group>
                <v-chip v-for="header in REQUIRED_HEADERS" :key="header" size="small" variant="flat" color="primary" class="font-weight-bold">{{ header }}</v-chip>
              </v-chip-group>
            </div>
            <div>
              <div class="text-caption font-weight-bold text-grey mb-1">OPTIONAL</div>
              <v-chip-group>
                <v-chip v-for="header in OPTIONAL_HEADERS" :key="header" size="small" variant="tonal" border>{{ header }}</v-chip>
              </v-chip-group>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<style scoped>
.survey-upload-view {
  width: 100%;
}

.premium-upload-card {
  border-radius: 32px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
}

.upload-zone {
  border: 2px dashed #e2e8f0;
  border-radius: 24px;
  padding: 60px 40px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s ease;
  background: #f8fafc;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.upload-zone:hover, .upload-zone.drag-over {
  border-color: #6366f1;
  background: #f1f5f9;
}

.upload-zone.drag-over {
  transform: scale(1.02);
}

.side-card {
  border-radius: 20px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
}

.border-bottom {
  border-bottom: 1px solid #f1f5f9;
}

.tracking-tight {
  letter-spacing: -0.025em;
}

.text-grey-darken-1 {
  color: #475569 !important;
}
</style>
