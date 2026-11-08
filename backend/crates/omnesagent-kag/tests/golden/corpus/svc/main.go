package main

import "fmt"

type Server struct {
	Name string
}

func (s *Server) Start() {
	fmt.Println(s.Name)
}

func main() {
	s := &Server{Name: "x"}
	s.Start()
}
