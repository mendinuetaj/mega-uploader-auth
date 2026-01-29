pipeline {
	agent {
		label 'k8s-agent'
	}
	environment {
		GIT_REPO = 'git@github.com:mendinuetaj/mega-uploader-auth.git'
		GIT_CREDENTIALS = 'github'
		DOCKER_IMAGE = 'mega-uploader-auth'
		DOCKER_TAG = "${env.GIT_TAG?:'latest'}"
	}
	stages {
		stage('Checkout') {
			steps {
				echo "Checking out..."
				checkout([
					$class: 'GitSCM',
					branches: [[name: "*/${env.GIT_TAG?:'master'}"]],
					userRemoteConfigs: [[
						url: env.GIT_REPO,
						credentialsId: env.GIT_CREDENTIALS
					]]
				])
			}
		}
		stage('Build Docker Image') {
			steps {
				container('docker') {
					script {
						echo "Building Docker image..."
						sh "docker build -t ${DOCKER_IMAGE}:${DOCKER_TAG} ."
					}
				}
			}
		}
		stage('Push Docker Image') {
			steps {
				container('docker') {
					echo "Pushing Docker image to Docker Hub..."
					withCredentials([usernamePassword(credentialsId: 'dockerhub', usernameVariable: 'DOCKERHUB_USER', passwordVariable: 'DOCKERHUB_PASS')]) {
						sh """
						echo \$DOCKERHUB_PASS | docker login -u \$DOCKERHUB_USER --password-stdin
						docker tag ${DOCKER_IMAGE}:${DOCKER_TAG} \$DOCKERHUB_USER/${DOCKER_IMAGE}:${DOCKER_TAG}
						docker push \$DOCKERHUB_USER/${DOCKER_IMAGE}:${DOCKER_TAG}
					"""
					}
				}
			}
		}
		stage('Deploy to K8s') {
			steps {
				container('kubectl') {
					withCredentials([usernamePassword(credentialsId: 'dockerhub', usernameVariable: 'DOCKERHUB_USER', passwordVariable: 'DOCKERHUB_PASS')]) {
						script {
							echo "Deploying to Kubernetes..."
							sh "kubectl apply -f k8s/namespace.yaml"
							sh "kubectl apply -f k8s/configmap.yaml"
							sh "envsubst < k8s/deployment.yaml | kubectl apply -f -"
							sh "kubectl apply -f k8s/service.yaml"
						}
					}
				}
			}
		}
	}
}