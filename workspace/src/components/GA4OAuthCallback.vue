<template>
  <div class="oauth-callback">
    <div class="callback-container">
      <div v-if="isLoading" class="loading-state">
        <div class="spinner"></div>
        <h1>Processing OAuth Connection...</h1>
        <p>Please wait while we complete your Google Analytics 4 connection.</p>
      </div>

      <div v-else-if="hasError" class="error-state">
        <div class="error-icon">⚠️</div>
        <h1>Connection Failed</h1>
        <p class="error-message">{{ errorMessage }}</p>
        <div class="error-details" v-if="errorCode">
          <span class="label">Error Code:</span>
          <span class="value">{{ errorCode }}</span>
        </div>
        <button @click="handleRetry" class="btn btn-primary">
          Try Again
        </button>
        <button @click="handleGoHome" class="btn btn-secondary">
          Back to Home
        </button>
      </div>

      <div v-else-if="showPropertySelection" class="property-selection-state">
        <div class="success-icon">✓</div>
        <h1>Connected Successfully!</h1>
        <p>Your Google Analytics 4 account has been connected.</p>
        <p>Now select a property to track:</p>

        <div class="properties-container">
          <div v-if="isLoadingProperties" class="loading-properties">
            Loading properties...
          </div>

          <div v-else-if="propertiesError" class="properties-error">
            {{ propertiesError }}
          </div>

          <div v-else-if="properties.length === 0" class="no-properties">
            No properties found.
          </div>

          <div v-else class="properties-list">
            <div
              v-for="property in properties"
              :key="property.name"
              @click="selectProperty(property)"
              :class="['property-item', { selected: selectedProperty?.name === property.name }]"
            >
              <div class="property-radio">
                <input
                  type="radio"
                  :name="property.name"
                  :checked="selectedProperty?.name === property.name"
                  @change="selectProperty(property)"
                />
              </div>
              <div class="property-info">
                <div class="property-display-name">{{ property.display_name }}</div>
                <div class="property-name">{{ property.name }}</div>
              </div>
            </div>
          </div>
        </div>

        <div class="actions">
          <button
            @click="handleConfirmProperty"
            :disabled="!selectedProperty || isSubmitting"
            class="btn btn-primary"
          >
            {{ isSubmitting ? 'Connecting...' : 'Confirm Property' }}
          </button>
          <button @click="handleGoHome" class="btn btn-secondary">
            Skip for Now
          </button>
        </div>
      </div>

      <div v-else-if="isSuccess" class="success-state">
        <div class="success-icon">✓</div>
        <h1>All Set!</h1>
        <p>Your Google Analytics 4 property has been successfully connected.</p>
        <div class="connection-info">
          <div class="info-item">
            <span class="label">Connector ID:</span>
            <code class="value">{{ connectionId }}</code>
          </div>
          <div class="info-item">
            <span class="label">Property:</span>
            <span class="value">{{ selectedProperty?.display_name }}</span>
          </div>
        </div>
        <button @click="handleGoHome" class="btn btn-primary">
          Go to Project Details
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import ga4Service from '@/services/ga4Service';
import type { GA4Property } from '@/types/ga4';

const isLoading = ref(true);
const isSuccess = ref(false);
const hasError = ref(false);
const showPropertySelection = ref(false);
const errorMessage = ref('');
const errorCode = ref('');
const connectionId = ref('');
const projectId = ref('');
const connectorId = ref('');

const properties = ref<GA4Property[]>([]);
const selectedProperty = ref<GA4Property | null>(null);
const isLoadingProperties = ref(false);
const propertiesError = ref('');
const isSubmitting = ref(false);

const handleRetry = () => {
  window.history.back();
};

const handleGoHome = () => {
  if (projectId.value) {
    window.location.href = `/?project=${projectId.value}`;
  } else {
    window.location.href = '/';
  }
};

const selectProperty = (property: GA4Property) => {
  selectedProperty.value = property;
};

const handleConfirmProperty = async () => {
  if (!selectedProperty.value || !projectId.value || !connectorId.value) return;

  isSubmitting.value = true;
  try {
    await ga4Service.selectProperty(
      projectId.value,
      connectorId.value,
      selectedProperty.value.display_name,
      selectedProperty.value.name
    );
    isSuccess.value = true;
    showPropertySelection.value = false;
  } catch (error) {
    propertiesError.value = error instanceof Error ? error.message : 'Failed to confirm property';
  } finally {
    isSubmitting.value = false;
  }
};

const fetchProperties = async () => {
  if (!projectId.value) return;

  isLoadingProperties.value = true;
  propertiesError.value = '';

  try {
    const fetchedProperties = await ga4Service.getProperties(projectId.value);
    properties.value = fetchedProperties;
    if (fetchedProperties.length > 0) {
      selectedProperty.value = fetchedProperties[0];
    }
  } catch (error) {
    propertiesError.value = error instanceof Error ? error.message : 'Failed to load properties';
  } finally {
    isLoadingProperties.value = false;
  }
};

onMounted(() => {
  const params = new URLSearchParams(window.location.search);

  // Extract parameters from URL
  const error = params.get('error');
  const errorDescription = params.get('error_description');
  const connId = params.get('connector_id');
  const projId = params.get('project_id');

  projectId.value = projId || '';
  connectorId.value = connId || '';
  connectionId.value = connId || '';

  setTimeout(() => {
    // Check for error parameter
    if (error) {
      hasError.value = true;
      errorCode.value = error;
      errorMessage.value = errorDescription || `OAuth connection failed: ${error}`;
      isLoading.value = false;
      return;
    }

    // Check for connection ID and project ID parameters
    if (connId && projId) {
      // Success - now fetch properties
      showPropertySelection.value = true;
      isLoading.value = false;
      fetchProperties();
      return;
    }

    // If we have connId but no projId, show error
    if (connId && !projId) {
      hasError.value = true;
      errorMessage.value = 'Missing project ID. Please try connecting again from the project details page.';
      isLoading.value = false;
      return;
    }

    // No error and no connection_id - unexpected state
    hasError.value = true;
    errorMessage.value = 'Unexpected response: No connection ID received';
    isLoading.value = false;
  }, 1000);
});
</script>

<style scoped>
.oauth-callback {
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.callback-container {
  background: white;
  border-radius: 12px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.1);
  padding: 60px 40px;
  max-width: 500px;
  width: 100%;
  text-align: center;
}

.loading-state,
.error-state,
.success-state,
.property-selection-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.spinner {
  width: 50px;
  height: 50px;
  border: 4px solid #f0f0f0;
  border-top: 4px solid #4a90e2;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

.loading-state h1,
.error-state h1,
.success-state h1,
.property-selection-state h1 {
  font-size: 28px;
  color: #333;
  margin: 0;
}

.loading-state p,
.error-state p,
.success-state p,
.property-selection-state p {
  font-size: 16px;
  color: #666;
  margin: 0;
}

.success-icon,
.error-icon {
  font-size: 60px;
  line-height: 1;
}

.success-icon {
  background: #e8f5e9;
  border-radius: 50%;
  width: 100px;
  height: 100px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #2e7d32;
}

.error-icon {
  background: #fff3e0;
  border-radius: 50%;
  width: 100px;
  height: 100px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #e65100;
}

.error-message {
  color: #e74c3c;
  font-weight: 500;
}

.error-details,
.connection-info {
  background: #f9f9f9;
  border-radius: 8px;
  padding: 16px;
  text-align: left;
  width: 100%;
}

.info-item,
.detail-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.info-item:last-child,
.detail-item:last-child {
  margin-bottom: 0;
}

.label {
  font-weight: 500;
  color: #666;
  font-size: 14px;
}

.value {
  background: white;
  border: 1px solid #e0e0e0;
  padding: 8px 12px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
  color: #333;
  word-break: break-all;
}

.properties-container {
  width: 100%;
  background: #f9f9f9;
  border-radius: 8px;
  padding: 16px;
  max-height: 400px;
  overflow-y: auto;
}

.properties-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.property-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.property-item:hover {
  border-color: #4a90e2;
  background-color: #f0f7ff;
}

.property-item.selected {
  border-color: #4a90e2;
  background-color: #f0f7ff;
}

.property-radio {
  display: flex;
  align-items: center;
  margin-top: 2px;
}

.property-radio input {
  cursor: pointer;
}

.property-info {
  flex: 1;
}

.property-display-name {
  font-weight: 500;
  color: #333;
  margin-bottom: 4px;
}

.property-name {
  font-size: 12px;
  color: #999;
  font-family: 'Courier New', monospace;
}

.loading-properties,
.properties-error,
.no-properties {
  padding: 20px;
  text-align: center;
  color: #666;
  font-size: 14px;
}

.properties-error {
  color: #e74c3c;
}

.actions {
  display: flex;
  gap: 12px;
  width: 100%;
  margin-top: 20px;
}

.btn {
  padding: 12px 24px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  flex: 1;
}

.btn-primary {
  background-color: #4a90e2;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #357abd;
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(74, 144, 226, 0.3);
}

.btn-primary:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.btn-secondary {
  background-color: #e0e0e0;
  color: #333;
}

.btn-secondary:hover {
  background-color: #d0d0d0;
  transform: translateY(-2px);
}

.loading-state button,
.error-state button,
.success-state button,
.property-selection-state button {
  margin-top: 10px;
}

@media (max-width: 640px) {
  .callback-container {
    padding: 40px 24px;
  }

  .loading-state h1,
  .error-state h1,
  .success-state h1,
  .property-selection-state h1 {
    font-size: 22px;
  }

  .properties-container {
    max-height: 300px;
  }

  .actions {
    flex-direction: column;
  }

  .btn {
    width: 100%;
  }
}
</style>
