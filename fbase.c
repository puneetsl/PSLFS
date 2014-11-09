#include<windows.h>
#include<stdio.h>
#include<conio.h>
#include<string.h>
#include<stdlib.h>

#include"partition.h"
#include"fstest1.h"
int main()
{
    ffolder current,tem;
    ffile fp,temp;
    int i;
    char name[32],command[100],p[10]="test1",path[10],path2[10],path3[10],path4[10],*contents;
    system("color f0");
    strcpy(path,p);
    strcat(path,".psl");
    
    strcpy(path2,p);
    strcat(path2,".fol");
    strcpy(path3,p);
    strcat(path3,".fil");
    strcpy(path4,p);
    strcat(path4,".fs");
    //install_pslos("test1.psl");
    while(Authenticate()!=0)
        printf("wrong password ... try again\n");
    
    psllogo();
    sleep(4);
    current=readFolder(path,1,current);
    invokeFreeSectors("test1.fol","test1.fil","test1.fs");
    while(1)
    {
        showFolder(path,current);
        //command=getCommand();
        gets(command);
        if(strncmp(command,"md ",3)==0)
        {
            //puts("in md");
            if(current.properties[1]=='w')
            {
                for(i=3;i<strlen(command);i++)
                    name[i-3]=command[i];
                name[i-3]=0;
                tem=activateFolder();
                strcpy(tem.name,name);
                writeFolder(path,current,tem);
                
                current=readFolder(path,current.sector,current);
            }
            else printf("Directory not writable!!\n");
        }
        if(strncmp(command,"cd ",3)==0)
        {
            //puts("in cd");
            for(i=3;i<strlen(command);i++)
                name[i-3]=command[i];
            name[i-3]=0;
            if(strcmp(name,"..")==0)
            {
                if(current.upsector!=0)
                    current=readFolder(path,current.upsector,current);
                
            }
            else if(strcmp(name,"/")==0)
            {
                current=readFolder(path,1,current);
            }
            else if(strcmp(name,"-p")==0)
            {
                current=readFolder(path,current.prev,current);
                if(current.properties[0]!='r')
                {
                    printf("this folder is not accessible\n");
                    current=readFolder(path,current.next,current);
                }
            }
            else if(strcmp(name,"-n")==0)
            {
                current=readFolder(path,current.next,current);
                if(current.properties[0]!='r')
                {
                    printf("this folder is not accessible\n");
                    current=readFolder(path,current.prev,current);
                }
            }
            else
            {
                int i=0,j=0;
                char tem[32];
                if(name[0]=='/')
                {
                    current=readFolder(path,1,current);
                    j=1;   }
                for(;;j++,i++)
                    
                {
                    
                    if(name[j]=='/')
                    {
                        tem[i]=0;
                        i=-1;
                        current=changeFolder(path,tem,current.sector);
                        current=readFolder(path,current.sector,current);
                    }
                    else if(name[j]==0)
                    {
                        tem[i]=0;
                        i=-1;
                        current=changeFolder(path,tem,current.sector);
                        current=readFolder(path,current.sector,current);
                        break;
                    }
                    else
                        tem[i]=name[j];
                }
            }
            
        }
        
        if(strcmp(command,"showdir")==0||strcmp(command,"ls")==0)
        {
            
            showdir(path,current);
            
            
        }
        if(strncmp(command,"show ",5)==0)
        {
            
            //puts("in cd");
            for(i=5;i<strlen(command);i++)
                name[i-5]=command[i];
            name[i-5]=0;
            showfile(path,current,name);
        }
        
        if(strncmp(command,"mf ",3)==0)
        {
            ffile *p;
            //puts("in cd");
            if(current.properties[1]=='w')
            {
                for(i=3;i<strlen(command);i++)
                    name[i-3]=command[i];
                name[i-3]=0;
                temp=activateFile();
                strcpy(temp.name,name);
                p=writeFile(path,current,temp);
                current=foldTravel(path,current.sector);
                system("edit alpha");
                
                if(p!=0)
                {
                    contents=intract("alpha");
                    unlink("alpha");
                    writeFileContents(path,contents,p);
                }
            }
            else printf("File not writable!!\n");
            system("color f0");
            
        }
        if(strncmp(command,"del ",4)==0)
        {
            //puts("in cd");
            for(i=4;i<strlen(command);i++)
                name[i-4]=command[i];
            name[i-4]=0;
            deletefold("test1.psl",current,name);
            current=readFolder(path,current.sector,current);
        }
        if(strncmp(command,"edit ",5)==0)
        {
            ffile *p;
            char g;
            printf("choose editor\n");
            printf("1.psleditor\n");
            printf("2.notepad\n");
            printf("3.cmd editor\n");
            g=getch();
            if(g=='1')
                system("editor alpha.txt");
            else if(g=='2')
                system("notepad alpha.txt");
            else
                system("edit alpha.txt");
            
            for(i=5;i<strlen(command);i++)
                name[i-5]=command[i];
            name[i-5]=0;
            extract("test1.psl",current,name);
            deletefile("test1.psl",current,name);
            
            temp=activateFile();
            strcpy(temp.name,name);
            p=writeFile(path,current,temp);
            if(p!=0)
            {
                contents=intract("alpha.txt");
                
                writeFileContents(path,contents,p);
                unlink("alpha.txt");
            }
            system("color f0");
        }
        if(strncmp(command,"df ",3)==0)
        {
            //puts("in cd");
            for(i=3;i<strlen(command);i++)
                name[i-3]=command[i];
            name[i-3]=0;
            deletefile("test1.psl",current,name);
            current=readFolder(path,current.sector,current);
        }
        if(strncmp(command,"cfmode ",7)==0)
        {
            
            for(i=7;i<strlen(command);i++)
                name[i-7]=command[i];
            name[i-7]=0;
            changeFilemode("test1.psl",current,name);
        }
        if(strncmp(command,"cdmode ",7)==0)
        {
            
            for(i=7;i<strlen(command);i++)
                name[i-7]=command[i];
            name[i-7]=0;
            changeFoldmode("test1.psl",current,name);
        }
        if(strcmp(command,"quit")==0)
        {
            puts("bye bye !! see you next time");
            getch();
            exit(1);
        }
        if(strcmp(command,"sudoku")==0)
        {
            system("color 0f");
            system("sudoku");
            system("color f0");
        }
        if(strcmp(command,"quark")==0)
        {
            system("quark");
            system("color f0");
        }
        if(strcmp(command,"clear")==0)
        {
            clrscr();
            system("color f0");
        }
        if(strcmp(command,"chat")==0)
        {
            system("chatpsl");
        }
        if(strncmp(command,"web ",4)==0)
        {
            
            for(i=4;i<strlen(command);i++)
                name[i-4]=command[i];
            name[i-4]=0;
            ShellExecute(0,"open",name, NULL, NULL, 0);
        }
        if(strcmp(command,"search")==0)
        {
            system("google");
            system("color f0");
        }
        
    }
    return 0;
}
