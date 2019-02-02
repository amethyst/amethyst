pipeline {
    agent {
        docker {
            image 'magnonellie/amethyst-dependencies:stable'
            label 'docker'
        } 
    }
    stages {
        stage('test') {
            steps {
                sh 'cargo test --all'
            }
        }
    }
}
