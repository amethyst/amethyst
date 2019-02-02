pipeline {
    agent {
        docker {
            image 'magnonellie/amethyst-dependencies:stable'
            label 'docker'
        } 
    }
    stages {
        stage('build') {
            steps {
                sh 'cargo build --all'
            }
        }
    }
}
